use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::{convert::Infallible, net::SocketAddr};
use std::{sync::Arc, time::Duration};

use axum::response::Redirect;
use axum_server::tls_rustls::RustlsConfig;
use tokio::signal;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_stream::{Stream, StreamExt as _};

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, Html, IntoResponse, Response, Sse},
    routing::get,
    Router,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const SYSTEM_REFRESH_PERIOD: Duration = Duration::from_secs(5);

struct AppState {
    system_tx: broadcast::Sender<Event>,
    system: Mutex<System>,
    kernel_version: Mutex<String>,
    disks: Mutex<Disks>,
    networks: Mutex<Networks>,
}

async fn send_system_messages(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(SYSTEM_REFRESH_PERIOD);
    loop {
        interval.tick().await;

        let uptime = System::uptime();

        let mut system = state.system.lock().await;
        system.refresh_processes(ProcessesToUpdate::All);
        let process_count = system.processes().len();

        let mut networks = state.networks.lock().await;
        networks.refresh();
        let mut total_rx = 0;
        let mut total_tx = 0;
        for (_interface_name, data) in networks.iter() {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }

        let event = Event::default().data(format!(
            "{total_rx:?}, {total_tx:?}, {process_count:?}, {uptime:?}"
        ));
        let _ = state.system_tx.send(event);
    }
}

#[tokio::main]
async fn main() {
    let exe_path = env::current_exe().expect("Could not get path of exe");
    let port_arg = std::env::args().nth(1).unwrap_or("3000".to_string());
    let port: u16 = port_arg.parse().unwrap_or(3000);

    let mut log_path;

    if cfg!(debug_assertions) {
        log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        log_path.push("debug.log");
    } else {
        log_path = exe_path.clone();
        log_path.pop();
        log_path.push("debug.log");
    }

    let log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path)
        .expect("Could not open ./debug.log");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(log_file),
        )
        .init();

    let mut certs_path;

    if cfg!(debug_assertions) {
        certs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        certs_path.push("certs");
    } else {
        certs_path = exe_path.clone();
        certs_path.pop();
        certs_path.push("certs");
    }

    tracing::info!("Loading TLS .pem files from {certs_path:?}");
    // configure certificate and private key used by https
    let config =
        RustlsConfig::from_pem_file(certs_path.join("cert.pem"), certs_path.join("key.pem"))
            .await
            .unwrap();

    // Create a new broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    // Create our shared state
    let state = Arc::new(AppState {
        system_tx: tx,
        system: Mutex::new(System::new_all()),
        kernel_version: Mutex::new(System::kernel_version().unwrap_or("v6.1".to_owned())),
        disks: Mutex::new(Disks::new_with_refreshed_list()),
        networks: Mutex::new(Networks::new_with_refreshed_list()),
    });

    // Create a handle for our TLS server so the shutdown signal can all shutdown
    let handle = axum_server::Handle::new();
    // Spawn a task to gracefully shutdown server.
    tokio::spawn(graceful_shutdown(handle.clone()));

    // Spawn a task to send events
    tokio::spawn(send_system_messages(state.clone()));

    // build our application with some routes
    // ? maybe use https://docs.rs/tower-default-headers/latest/tower_default_headers/ to add 'server: Axum' header
    let app = Router::new()
        .fallback(get(|| async { Redirect::permanent("/") }))
        .route("/", get(index_handler))
        .route("/sse", get(sse_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting HTTPS server at {addr}");
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();

    tracing::info!("Server has been shut down.");
}

// https://github.com/tokio-rs/axum/blob/1ac617a1b540e8523347f5ee889d65cad9a45ec4/examples/tls-graceful-shutdown/src/main.rs
// https://github.com/programatik29/axum-server/blob/d48b1a931909d156177bc87684910769e67be905/examples/graceful_shutdown.rs
async fn graceful_shutdown(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Received termination signal, shutting down...");
    // Refuses new connections
    // 10 secs is how long docker will wait to force shutdown
    handle.graceful_shutdown(Some(Duration::from_secs(10)));

    // Print alive connection count every second.
    loop {
        sleep(Duration::from_secs(1)).await;

        tracing::debug!("alive connections: {}", handle.connection_count());
    }
}

async fn sse_handler(
    state: State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let system_rx = state.system_tx.subscribe();

    let system_stream = tokio_stream::wrappers::BroadcastStream::new(system_rx)
        .map(|msg| msg.unwrap_or_else(|_| Event::default().data("error")))
        .map(Ok);

    Sse::new(system_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn index_handler(state: State<Arc<AppState>>) -> impl IntoResponse {
    // TODO: Get model name from `tail -n 1 /proc/cpuinfo | cut -d':' -f2 | cut -c2-`
    let system = state.system.lock().await;
    let cpu_brand = system.cpus()[0].brand();
    let disks = state.disks.lock().await;
    let networks = state.networks.lock().await;

    let mut total_rx = 0;
    let mut total_tx = 0;
    for (_interface_name, data) in networks.iter() {
        total_rx += data.total_received();
        total_tx += data.total_transmitted();
    }

    let template = IndexTemplate {
        kernel_version: state.kernel_version.lock().await.to_string(), // 6.6.31+rpt-rpi-v8
        model_name: "Raspberry Pi 4 Model B Rev 1.4".to_owned(),
        cpu_brand: cpu_brand.to_string(), // Cortex-A72
        cpu_brand_short: cpu_brand[0..cpu_brand.len() - 2].to_string().to_uppercase(), // CORTEX-A
        cpu_count: system.cpus().len(),   // 4
        cpu_speed: system.cpus()[0].frequency(), // 1800 MHz
        extended_memory: (system.total_memory() - (1_048_576_000)) / 1_000, // 4 GB
        primary_disk_size: (disks[0].total_space() / 1_000_000_000 + 7) & !7, // 32 GB
        total_memory: system.total_memory(), // 4 GB
        rounded_memory: (system.total_memory() / 1_000_000_000 + 3) & !3, // 4 GB
        uptime: System::uptime().to_string(),
        process_count: system.processes().len(),
        rx: total_rx,
        tx: total_tx,
    };

    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    kernel_version: String,
    model_name: String,
    cpu_brand: String,
    cpu_brand_short: String,
    cpu_count: usize,
    cpu_speed: u64,
    extended_memory: u64,
    primary_disk_size: u64,
    total_memory: u64,
    rounded_memory: u64,
    uptime: String,
    process_count: usize,
    rx: u64,
    tx: u64,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
