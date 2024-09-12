use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::ExitCode;
use std::{convert::Infallible, net::SocketAddr};
use std::{sync::Arc, time::Duration};

use axum::extract::{ConnectInfo, Host};
use axum::handler::HandlerWithoutStateExt;
use axum::response::sse::KeepAlive;
use axum::response::Redirect;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use tokio::signal;
use tokio::sync::{watch, Mutex};
use tokio::time::sleep;
use tokio_stream::wrappers::WatchStream;
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

#[derive(Parser, Debug)]
#[command(version = env!("RSPI_BIOS_VERSION"), about)]
struct Args {
    #[arg(long, value_parser = parse_duration, default_value = "5")]
    system_refresh_interval: Duration,

    #[arg(long, value_parser = parse_duration, default_value = "10")]
    graceful_shutdown_duration: Duration,

    #[arg(long, value_parser = parse_duration, default_value = "1")]
    alive_connections_check_interval: Duration,

    #[arg(long, value_parser = parse_duration, default_value = "1")]
    sse_keep_alive_interval: Duration,

    #[arg(long)]
    https_redirect: bool,

    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    ip_address: std::net::IpAddr,

    #[arg(long, default_value_t = 2000)]
    http_port: u16,

    #[arg(long, default_value_t = 3000)]
    https_port: u16,

    #[arg(long, default_value_os_t = PathBuf::from("/etc/rspi-bios/certs"))]
    tls_dir: PathBuf,

    #[arg(long, default_value = "cert.pem")]
    tls_cert_file_name: String,

    #[arg(long, default_value = "key.pem")]
    tls_key_file_name: String,

    #[arg(long, default_value_os_t = PathBuf::from("/var/log/rspi-bios/"))]
    log_path: PathBuf,

    #[arg(long, default_value = "0, 0, 0, 0")]
    system_stream_error_data: String,

    #[arg(long, default_value = "v6.1")]
    kernel_version_fallback: String,

    #[arg(long, default_value = "Cortex-A72")]
    cpu_brand_fallback: String,

    #[arg(long, default_value = "Cortex-A")]
    cpu_brand_short_fallback: String,

    #[arg(long, default_value_t = 1_800)]
    cpu_frequency_fallback: u64,

    #[arg(long, default_value_t = 32_000_000_000)]
    disk_space_fallback: u64,

    #[arg(long, default_value = "Raspberry Pi 4 Model B Rev 1.4")]
    model_name_fallback: String,

    #[arg(long, default_value = "Raspbian GNU/Linux 11 (bullseye)")]
    os_version_fallback: String,

    #[arg(long, default_value = "aarch64")]
    cpu_arch_fallback: String,

    /// Send DEBUG events to STDOUT in release
    #[arg(long)]
    force_debug_stdout: bool,

    /// Place debug log file in the same directory as the binary (overrides `--log_path`)
    #[arg(long)]
    force_debug_local: bool,
}

// https://stackoverflow.com/a/72314001/19020549
fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(Duration::from_secs(seconds))
}

struct AppState {
    args: Mutex<Args>,
    system_tx: watch::Sender<Event>,
    system: Mutex<System>,
    kernel_version: Mutex<String>,
    os_version: Mutex<String>,
    cpu_arch: Mutex<String>,
    disks: Mutex<Disks>,
    networks: Mutex<Networks>,
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let exe_path = match env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to get exe path.\n\nError={e:#?}");
            return ExitCode::FAILURE;
        }
    };
    let log_path = get_log_path(&exe_path, &args.log_path, args.force_debug_local);

    let log_file_result = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path);

    let log_file = match log_file_result {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to open/create {log_path:?}, did you set the correct permissions?\n\nError={e:#?}");
            return ExitCode::FAILURE;
        }
    };

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

    if cfg!(debug_assertions) || args.force_debug_stdout {
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
    tracing::debug!(
        "Running {} version {}",
        env!("CARGO_CRATE_NAME"),
        env!("RSPI_BIOS_VERSION")
    );

    tracing::info!("Creating TLS config");
    let cert_dirs_to_search = get_cert_dirs_to_search(&exe_path, &args.tls_dir);
    let Some(tls_config) = create_tls_config(
        cert_dirs_to_search,
        &args.tls_cert_file_name,
        &args.tls_key_file_name,
    )
    .await
    else {
        tracing::error!("Failed to create TLS config, did you set the correct permissions? Did you put the .pem files in the correct place?");
        return ExitCode::FAILURE;
    };

    // Create a handle for our TLS server so the shutdown signal can all shutdown
    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tracing::debug!("Spawning graceful shutdown handler");
    let graceful_shutdown_task = tokio::spawn(graceful_shutdown(
        handle.clone(),
        args.graceful_shutdown_duration,
        args.alive_connections_check_interval,
    ));

    let https_redirect_task = if args.https_redirect {
        // Spawn a second server to redirect http requests to this server
        tokio::spawn(redirect_http_to_https(
            args.ip_address,
            args.http_port,
            args.https_port,
        ))
    } else {
        // TODO: Find out if there is a better way than doing this
        tokio::spawn(async {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
            }
        })
    };

    let addr = SocketAddr::from((args.ip_address, args.https_port));

    let tx = watch::Sender::new(Event::default().data(&args.system_stream_error_data));

    // Create our shared state
    tracing::debug!("Creating initial state");
    let state = Arc::new(AppState {
        kernel_version: Mutex::new(
            System::kernel_version().unwrap_or_else(|| args.kernel_version_fallback.clone()),
        ),
        os_version: Mutex::new(
            System::long_os_version().unwrap_or_else(|| args.os_version_fallback.clone()),
        ),
        cpu_arch: Mutex::new(System::cpu_arch().unwrap_or_else(|| args.cpu_arch_fallback.clone())),
        args: Mutex::new(args),
        system_tx: tx,
        system: Mutex::new(System::new_all()),
        disks: Mutex::new(Disks::new_with_refreshed_list()),
        networks: Mutex::new(Networks::new_with_refreshed_list()),
    });

    // Spawn a task to send events
    tracing::debug!("Spawning system info stream");
    let system_messages_task = tokio::spawn(send_system_messages(state.clone()));

    // Spawn main server
    let https_server_task = tokio::spawn(https_server(addr, state, tls_config, handle));

    // Wait for a task to complete, useful if 'https_redirect_task' returns error for example.
    tokio::select! {
        _ = graceful_shutdown_task => {},
        result = https_redirect_task => {
            match result {
                Ok(s) => {
                    if !s {
                        return ExitCode::FAILURE;
                    }
                },
                Err(_) => {
                    return ExitCode::FAILURE
                },
            }
        },
        _ = system_messages_task => {},
        result = https_server_task => {
            match result {
                Ok(s) => {
                    if !s {
                        return ExitCode::FAILURE;
                    }
                },
                Err(_) => {
                    return ExitCode::FAILURE
                },
            }
        },
    }

    tracing::info!("Goodbye");
    ExitCode::SUCCESS
}

/// Returns `false` on failure
async fn https_server(
    addr: std::net::SocketAddr,
    state: Arc<AppState>,
    tls_config: RustlsConfig,
    handle: axum_server::Handle,
) -> bool {
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

    tracing::info!("Starting HTTPS server at {addr}");
    let axum_result = axum_server::bind_rustls(addr, tls_config)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await;

    if let Err(e) = axum_result {
        tracing::error!(error = %e, "Failed to start HTTPS server at {addr}, did you set the correct permissions?");
        return false;
    };

    true
}

// https://github.com/tokio-rs/axum/blob/6efcb75d99a437fa80c81e2308ec8234b023e1a7/examples/tls-rustls/src/main.rs
/// Returns `false` on failure
#[allow(clippy::similar_names)]
async fn redirect_http_to_https(
    ip_address: std::net::IpAddr,
    http_port: u16,
    https_port: u16,
) -> bool {
    fn make_https(
        host: &str,
        uri: axum::http::Uri,
        http_port: u16,
        https_port: u16,
    ) -> Result<axum::http::Uri, axum::BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some(axum::http::uri::PathAndQuery::from_static("/"));
        }

        let https_host = host.replace(&http_port.to_string(), &https_port.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(axum::http::Uri::from_parts(parts)?)
    }

    let redirect = move |ConnectInfo(addr): ConnectInfo<SocketAddr>,
                         Host(host): Host,
                         uri: axum::http::Uri| async move {
        tracing::debug!("Connection made to HTTPS redirect server from {addr}");

        match make_https(&host, uri.clone(), http_port, https_port) {
            Ok(uri) => {
                tracing::info!("Redirecting {addr} to HTTPS");
                Ok(Redirect::permanent(&uri.to_string()))
            }
            Err(e) => {
                tracing::warn!(error = %e, uri = %uri, "Failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from((ip_address, http_port));

    tracing::info!("Starting HTTP redirect server at {addr}");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(error = %e, "Failed to start HTTPS redirect server at {addr}, did you set the correct permissions?");
            return false;
        }
    };

    let axum_result = axum::serve(
        listener,
        redirect.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await;

    if let Err(e) = axum_result {
        tracing::error!(error = %e, "Failed to start HTTPS redirect server at {addr}, did you set the correct permissions?");
        return false;
    };

    true
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
    version: String,
    os_version: String,
    cpu_arch: String,
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

    let (
        cpu_brand_fallback,
        cpu_frequency_fallback,
        disk_space_fallback,
        model_name_fallback,
        cpu_brand_short_fallback,
    ) = {
        let args = state.args.lock().await;
        (
            args.cpu_brand_fallback.clone(),
            args.cpu_frequency_fallback,
            args.disk_space_fallback,
            args.model_name_fallback.clone(),
            args.cpu_brand_short_fallback.clone(),
        )
    };

    let (cpu_brand, cpu_count, cpu_speed, total_memory, process_count) = {
        let system = state.system.lock().await;
        (
            system
                .cpus()
                .first()
                .map_or_else(|| cpu_brand_fallback.to_string(), |c| c.brand().to_string()),
            system.cpus().len(),
            system
                .cpus()
                .first()
                .map_or(cpu_frequency_fallback, sysinfo::Cpu::frequency),
            system.total_memory(),
            system.processes().len(),
        )
    };

    let primary_disk_size = {
        let disks = state.disks.lock().await;
        (disks
            .first()
            .map_or(disk_space_fallback, sysinfo::Disk::total_space)
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
        model_name: model_name_fallback.to_string(),
        cpu_brand: cpu_brand.to_string(), // Cortex-A72
        cpu_brand_short: cpu_brand
            .get(0..cpu_brand.len() - 2)
            .unwrap_or(&cpu_brand_short_fallback)
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
        version: env!("RSPI_BIOS_VERSION").to_string(),
        os_version: state.os_version.lock().await.to_string(),
        cpu_arch: state.cpu_arch.lock().await.to_string(),
    };

    HtmlTemplate(template)
}

async fn sse_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Connection made to SSE from {addr}");

    let system_rx = state.system_tx.subscribe();

    let system_stream = WatchStream::from_changes(system_rx).map(Ok);

    Sse::new(system_stream)
        .keep_alive(KeepAlive::new().interval(state.args.lock().await.sse_keep_alive_interval))
}

async fn send_system_messages(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(state.args.lock().await.system_refresh_interval);
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
async fn graceful_shutdown(
    handle: axum_server::Handle,
    graceful_shutdown_duration: Duration,
    alive_connections_check_interval: Duration,
) {
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
    handle.graceful_shutdown(Some(graceful_shutdown_duration));

    // Print alive connection count every second.
    loop {
        sleep(alive_connections_check_interval).await;
        tracing::debug!("Alive connections: {}", handle.connection_count());
    }
}

async fn create_tls_config(
    cert_dirs_to_search: Vec<PathBuf>,
    tls_cert_file_name: &str,
    tls_key_file_name: &str,
) -> Option<RustlsConfig> {
    for cert_dir in &cert_dirs_to_search {
        tracing::debug!("Attempting to load TLS .pem files from {cert_dir:?}");
        let config_result = RustlsConfig::from_pem_file(
            cert_dir.join(tls_cert_file_name),
            cert_dir.join(tls_key_file_name),
        )
        .await;

        match config_result {
            Ok(t) => {
                tracing::info!("Found TLS {tls_cert_file_name} and {tls_key_file_name} file(s) in {cert_dir:?}");
                return Some(t);
            }
            Err(e) => {
                tracing::debug!(error = %e, "Failed to read/find TLS {tls_cert_file_name} and/or {tls_key_file_name} file(s) in {cert_dir:?}");
            }
        }
    }

    None
}

fn get_cert_dirs_to_search(
    exe_path: &std::path::Path,
    tls_dir: &std::path::Path,
) -> std::vec::Vec<std::path::PathBuf> {
    let mut cert_dirs_to_search = Vec::<PathBuf>::new();

    if cfg!(debug_assertions) {
        let cargo_certs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("certs");
        cert_dirs_to_search.push(cargo_certs_path);
    }

    #[cfg(unix)]
    {
        cert_dirs_to_search.push(tls_dir.to_path_buf());
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

fn get_log_path(
    exe_path: &std::path::Path,
    log_path_arg: &std::path::Path,
    force_debug_local: bool,
) -> std::path::PathBuf {
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
    } else if cfg!(windows) || force_debug_local {
        exe_log_path.clone()
    } else {
        let mut log_path = log_path_arg.to_path_buf();
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
