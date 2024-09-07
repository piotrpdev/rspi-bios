use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::{convert::Infallible, net::SocketAddr};
use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};

use axum::extract::ConnectInfo;
use axum::response::sse::KeepAlive;
use axum::response::Redirect;
use axum_server::tls_rustls::RustlsConfig;
use tokio::signal;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_stream::wrappers::BroadcastStream;
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
use tracing::Level;
use tracing_subscriber::{filter, Layer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const SYSTEM_REFRESH_PERIOD: Duration = Duration::from_secs(5);
const GRACEFUL_SHUTDOWN_PERIOD: Duration = Duration::from_secs(10);
const ALIVE_CONNECTIONS_CHECK_PERIOD: Duration = Duration::from_secs(1);
const SSE_KEEP_ALIVE_PERIOD: Duration = Duration::from_secs(1);

const DEFAULT_IP_ADDRESS: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const DEFAULT_PORT: u16 = 3000;

const DEFAULT_TLS_DIR: &str = "/etc/rspi-bios/certs";
const DEFAULT_TLS_CERT_FILE_NAME: &str = "cert.pem";
const DEFAULT_TLS_KEY_FILE: &str = "key.pem";

const DEFAULT_LOG_PATH: &str = "/var/log/rspi-bios/";

const SYSTEM_STREAM_ERROR_DATA: &str = "Error occurred while attempting to process system stream";

const DEFAULT_KERNEL_VERSION: &str = "v6.1";
const DEFAULT_CPU_BRAND: &str = "Cortex-A72";
const DEFAULT_CPU_BRAND_SHORT: &str = "Cortex-A";
const DEFAULT_CPU_FREQUENCY: u64 = 1_800;
const DEFAULT_DISK_SPACE: u64 = 32_000_000_000;
const DEFAULT_MODEL_NAME: &str = "Raspberry Pi 4 Model B Rev 1.4";

struct AppState {
    system_tx: broadcast::Sender<Event>,
    system: Mutex<System>,
    kernel_version: Mutex<String>,
    disks: Mutex<Disks>,
    networks: Mutex<Networks>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let exe_path = env::current_exe().context("Failed to get exe path")?;
    let port = env::args().nth(1).map_or(DEFAULT_PORT, |port_arg| {
        port_arg.parse().unwrap_or(DEFAULT_PORT)
    });

    let log_path = get_log_path(&exe_path);

    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path.clone())
        .with_context(|| format!("Failed to open/create {log_path:?}"))?;

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(log_file),
        );

    if cfg!(debug_assertions) || env::var("RSPI_BIOS_DEBUG").is_ok() {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    } else {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(filter::LevelFilter::from_level(Level::INFO)),
            )
            .init();
    }
    tracing::info!("Logging to {log_path:?}");

    tracing::info!("Creating TLS config");
    let cert_dirs_to_search = get_cert_dirs_to_search(&exe_path);
    let config = create_tls_config(cert_dirs_to_search)
        .await
        .context("Failed to create TLS config")?;

    // Create a new broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    // Create our shared state
    tracing::debug!("Creating initial state");
    let state = Arc::new(AppState {
        system_tx: tx,
        system: Mutex::new(System::new_all()),
        kernel_version: Mutex::new(
            System::kernel_version().unwrap_or_else(|| DEFAULT_KERNEL_VERSION.to_string()),
        ),
        disks: Mutex::new(Disks::new_with_refreshed_list()),
        networks: Mutex::new(Networks::new_with_refreshed_list()),
    });

    // Create a handle for our TLS server so the shutdown signal can all shutdown
    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tracing::debug!("Spawning graceful shutdown handler");
    tokio::spawn(graceful_shutdown(handle.clone()));

    // Spawn a task to send events
    tracing::debug!("Spawning system info stream");
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

    let addr = SocketAddr::from((DEFAULT_IP_ADDRESS, port));

    tracing::info!("Starting HTTPS server at {addr}");
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .with_context(|| format!("Failed to start HTTPS server at {addr}"))?;

    tracing::info!("Server has been shut down.");

    Ok(())
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

async fn index_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("Connection made to index.html from {addr}");
    // TODO: Get model name from `tail -n 1 /proc/cpuinfo | cut -d':' -f2 | cut -c2-`

    let (cpu_brand, cpu_count, cpu_speed, total_memory, process_count) = {
        let system = state.system.lock().await;
        (
            system
                .cpus()
                .first()
                .map_or_else(|| DEFAULT_CPU_BRAND.to_string(), |c| c.brand().to_string()),
            system.cpus().len(),
            system
                .cpus()
                .first()
                .map_or(DEFAULT_CPU_FREQUENCY, sysinfo::Cpu::frequency),
            system.total_memory(),
            system.processes().len(),
        )
    };

    let primary_disk_size = {
        let disks = state.disks.lock().await;
        (disks
            .first()
            .map_or(DEFAULT_DISK_SPACE, sysinfo::Disk::total_space)
            / 1_000_000_000
            + 7)
            & !7
    };

    let mut total_rx = 0;
    let mut total_tx = 0;
    {
        let networks = state.networks.lock().await;
        for (_interface_name, data) in networks.iter() {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }
    }

    let template = IndexTemplate {
        kernel_version: state.kernel_version.lock().await.to_string(), // 6.6.31+rpt-rpi-v8
        model_name: DEFAULT_MODEL_NAME.to_string(),
        cpu_brand: cpu_brand.to_string(), // Cortex-A72
        cpu_brand_short: cpu_brand
            .get(0..cpu_brand.len() - 2)
            .unwrap_or(DEFAULT_CPU_BRAND_SHORT)
            .to_string()
            .to_uppercase(), // CORTEX-A
        cpu_count,                        // 4
        cpu_speed,                        // 1800 MHz
        extended_memory: (total_memory - (1_048_576_000)) / 1_000, // 4 GB
        primary_disk_size,                // 32 GB
        total_memory,                     // 4 GB
        rounded_memory: (total_memory / 1_000_000_000 + 3) & !3, // 4 GB
        uptime: System::uptime().to_string(),
        process_count,
        rx: total_rx,
        tx: total_tx,
    };

    HtmlTemplate(template)
}

async fn sse_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Connection made to SSE from {addr}");

    let system_rx = state.system_tx.subscribe();

    let system_stream = BroadcastStream::new(system_rx)
        .map(|msg| msg.unwrap_or_else(|_| Event::default().data(SYSTEM_STREAM_ERROR_DATA)))
        .map(Ok);

    Sse::new(system_stream).keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE_PERIOD))
}

async fn send_system_messages(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(SYSTEM_REFRESH_PERIOD);
    loop {
        interval.tick().await;

        let uptime = System::uptime();

        let process_count = {
            let mut system = state.system.lock().await;
            system.refresh_processes(ProcessesToUpdate::All);
            system.processes().len()
        };

        let mut total_rx = 0;
        let mut total_tx = 0;
        {
            let mut networks = state.networks.lock().await;
            networks.refresh();

            for (_interface_name, data) in networks.iter() {
                total_rx += data.total_received();
                total_tx += data.total_transmitted();
            }
        };

        let event = Event::default().data(format!(
            "{total_rx:?}, {total_tx:?}, {process_count:?}, {uptime:?}"
        ));
        let _ = state.system_tx.send(event);
    }
}

// https://github.com/tokio-rs/axum/blob/1ac617a1b540e8523347f5ee889d65cad9a45ec4/examples/tls-graceful-shutdown/src/main.rs
// https://github.com/programatik29/axum-server/blob/d48b1a931909d156177bc87684910769e67be905/examples/graceful_shutdown.rs
async fn graceful_shutdown(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to install Ctrl+C handler");
        });
    };

    #[cfg(unix)]
    let terminate = async {
        let signal_result = signal::unix::signal(signal::unix::SignalKind::terminate());
        match signal_result {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => tracing::warn!(error = %e, "Failed to install Unix SIGTERM signal handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    // Refuses new connections
    // 10 secs is how long docker will wait to force shutdown
    tracing::info!("Received termination signal, shutting down...");
    handle.graceful_shutdown(Some(GRACEFUL_SHUTDOWN_PERIOD));

    // Print alive connection count every second.
    loop {
        sleep(ALIVE_CONNECTIONS_CHECK_PERIOD).await;
        tracing::debug!("Alive connections: {}", handle.connection_count());
    }
}

async fn create_tls_config(cert_dirs_to_search: Vec<PathBuf>) -> Option<RustlsConfig> {
    for cert_dir in &cert_dirs_to_search {
        tracing::debug!("Attempting to load TLS .pem files from {cert_dir:?}");
        let config_result = RustlsConfig::from_pem_file(
            cert_dir.join(DEFAULT_TLS_CERT_FILE_NAME),
            cert_dir.join(DEFAULT_TLS_KEY_FILE),
        )
        .await;

        match config_result {
            Ok(t) => {
                tracing::info!("Found TLS {DEFAULT_TLS_CERT_FILE_NAME} and {DEFAULT_TLS_KEY_FILE} file(s) in {cert_dir:?}");
                return Some(t);
            }
            Err(e) => {
                tracing::debug!(error = %e, "Failed to read/find TLS {DEFAULT_TLS_CERT_FILE_NAME} and/or {DEFAULT_TLS_KEY_FILE} file(s) in {cert_dir:?}");
            }
        }
    }

    None
}

fn get_cert_dirs_to_search(exe_path: &std::path::Path) -> std::vec::Vec<std::path::PathBuf> {
    let mut cert_dirs_to_search = Vec::<PathBuf>::new();

    if cfg!(debug_assertions) || env::var("RSPI_BIOS_DEBUG").is_ok() {
        let cargo_certs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("certs");
        cert_dirs_to_search.push(cargo_certs_path);
    }

    #[cfg(unix)]
    {
        let etc_certs_path = PathBuf::from(DEFAULT_TLS_DIR);
        cert_dirs_to_search.push(etc_certs_path);
    }

    let local_certs_path = {
        let mut certs_path = exe_path.to_path_buf();
        certs_path.pop();
        certs_path.push("certs");
        certs_path
    };
    cert_dirs_to_search.push(local_certs_path);

    cert_dirs_to_search
}

fn get_log_path(exe_path: &std::path::Path) -> std::path::PathBuf {
    let exe_log_path = {
        let mut log_path = exe_path.to_path_buf();
        log_path.pop();
        log_path.push("debug.log");
        log_path
    };

    let mut log_path = if cfg!(debug_assertions) {
        let mut log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        log_path.push("debug.log");
        log_path
    } else if cfg!(windows) || env::var("RSPI_BIOS_DEBUG_LOCAL_LOG").is_ok() {
        exe_log_path.clone()
    } else {
        let mut log_path = PathBuf::from(DEFAULT_LOG_PATH);
        log_path.push("debug.log");
        log_path
    };

    let mut parent = log_path.clone();
    parent.pop();
    let create_dir_result = std::fs::create_dir_all(parent);

    if let Err(e) = create_dir_result {
        eprintln!("Failed to create parent dirs for {log_path:?}, using {exe_log_path:?} instead. Error: {e:?}");
        log_path = exe_log_path;
    };

    log_path
}
