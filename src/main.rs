use axum::{
    extract::State,
    response::{sse::Event, Sse},
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::stream::Stream;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use sysinfo::System;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::StreamExt as _;
use tower_livereload::LiveReloadLayer;

use std::{
    convert::Infallible,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const SYSTEM_REFRESH_PERIOD: Duration = Duration::from_secs(1);

struct AppState {
    system_tx: broadcast::Sender<Event>,
    system: Mutex<System>,
}

async fn send_system_messages(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(SYSTEM_REFRESH_PERIOD);
    loop {
        interval.tick().await;
        let mut system = state.system.lock().await;
        system.refresh_all();
        let event = Event::default().data(format!("{system:?}"));
        let _ = state.system_tx.send(event);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let web_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/");

    // Create a new broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    // Create our shared state
    let state = Arc::new(AppState {
        system_tx: tx,
        system: Mutex::new(System::new_all()),
    });

    // Spawn a task to send events
    tokio::spawn(send_system_messages(state.clone()));

    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    // build our application with some routes
    // ? maybe use https://docs.rs/tower-default-headers/latest/tower_default_headers/ to add 'server: Axum' header
    let app = Router::new()
        .fallback_service(ServeDir::new(web_dir).append_index_html_on_directories(true))
        .route("/sse", get(sse_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state)
        .layer(livereload);

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        move |res: DebounceEventResult| match res {
            Ok(_) => reloader.reload(),
            Err(e) => tracing::error!("Watcher (debouncer) Error {:?}", e),
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(Path::new("./assets"), RecursiveMode::Recursive)
        .unwrap();

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    state: State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let system_rx = state.system_tx.subscribe();

    println!("`{}` connected", user_agent.as_str());

    // wrap using tokio_stream::wrappers::BroadcastStream
    let system_stream = tokio_stream::wrappers::BroadcastStream::new(system_rx)
        .map(|msg| msg.unwrap_or_else(|_| Event::default().data("error")))
        .map(Ok);

    Sse::new(system_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
