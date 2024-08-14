use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use sysinfo::System;
use tokio::sync::{broadcast, Mutex};

use std::{sync::Arc, time::Duration};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
 }

const SYSTEM_REFRESH_PERIOD: Duration = Duration::from_secs(1);
const PKG_VERSION: &str = built_info::PKG_VERSION;

struct AppState {
    system_tx: broadcast::Sender<Message>,
    system: Mutex<System>,
}

async fn send_system_ws_messages(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(SYSTEM_REFRESH_PERIOD);
    loop {
        interval.tick().await;
        let mut system = state.system.lock().await;
        system.refresh_all();
        let event = Message::Text(format!("{:?}", system));
        let _ = state.system_tx.send(event);
    }
}

// TODO: send built info
// TODO: send versions of the CI
// TODO: send versions of rust, npm, etc. use npm built equivalent
// TODO: highlight rust/axum process in the list of processes
// TODO: switch to using JS framework for the frontend
// TODO: setup hot reloading for front and back end
// TODO: optimize perf
// TODO: make build.rs build the frontend
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let web_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web/dist/");

    // Create a new broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    // Create our shared state
    let state = Arc::new(AppState {
        system_tx: tx,
        system: Mutex::new(System::new_all()),
    });

    // Spawn a task to send events
    tokio::spawn(send_system_ws_messages(state.clone()));

    // build our application with some routes
    // ? maybe use https://docs.rs/tower-default-headers/latest/tower_default_headers/ to add 'server: Axum' header
    let app = Router::new()
        .fallback_service(ServeDir::new(web_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state);

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

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, State(state): State<Arc<AppState>>) {
    // Subscribe to the broadcast channel
    let mut rx = state.system_tx.subscribe();

    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Text(PKG_VERSION.to_owned())).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Sent built info {who}...");
    } else {
        println!("Could not send built info {who}!");
        return;
    }

    loop {
        match rx.recv().await {
            Ok(event) => {
                if socket.send(event).await.is_ok() {
                    println!("Sent message to {who}...");
                } else {
                    println!("Could not send message to {who}!");
                    break;
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                println!("Channel closed for {who}!");
                break;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => {
                println!("Channel lagged for {who}!");
                break;
            }
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}
