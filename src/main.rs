use axum::{
    body::Body,
    extract::{Path, Query, State},
    routing::{get, post},
    Router,
};
use clap::Parser;
use http::StatusCode;
use proxy::proxy_request;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use subproc::WrappedCommand;
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener};
use tokio_stream::StreamExt;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod auth;
mod logging;
mod proxy;
mod subproc;

#[derive(Parser)]
struct Cli {
    /// Subserver command to run.
    #[arg(short, long)]
    command: WrappedCommand,

    /// Port to listen on
    #[arg(long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "9090")]
    subprocess_port: u16,
}

pub struct ServerState {
    wrapped_server: Arc<subproc::WrappedServer>,
    subprocess_port: u16,
}

type ArcServerState = Arc<ServerState>;

async fn status(State(state): State<ArcServerState>) -> (StatusCode, String) {
    if state.wrapped_server.running() {
        (StatusCode::OK, "Running.".into())
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Not running.".into())
    }
}

async fn restart(State(state): State<ArcServerState>) -> (StatusCode, String) {
    state.wrapped_server.restart().await;
    (StatusCode::OK, "Server restarting.".into())
}

async fn interrupt(State(state): State<ArcServerState>) -> (StatusCode, String) {
    state.wrapped_server.interrupt().await;
    (StatusCode::OK, "Server interrupted.".into())
}

#[derive(Deserialize)]
struct UploadQuery {
    #[serde(default)]
    restart: bool,
    #[serde(default)]
    interrupt: bool,
}

async fn upload(
    State(state): State<ArcServerState>,
    Query(query): Query<UploadQuery>,
    Path(filename): Path<String>,
    body: Body,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let mut body = body.into_data_stream();

    let path = PathBuf::from(filename);
    let parent_dir = path.parent();
    if let Some(parent_dir) = parent_dir {
        tokio::fs::create_dir_all(parent_dir).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Open file
    let mut file = File::create(&path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create file: {}", e),
        )
    })?;

    // Write file from stream
    while let Some(chunk) = body.next().await {
        file.write_all(&chunk.unwrap()).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write file: {}", e),
            )
        })?;
    }

    if query.restart {
        state.wrapped_server.restart().await;
    } else if query.interrupt {
        state.wrapped_server.interrupt().await;
    } else {
        state.wrapped_server.state_change().await;
    }

    Ok((
        StatusCode::OK,
        format!("File uploaded successfully to {}", path.display()),
    ))
}

async fn wait_for_reload(State(state): State<ArcServerState>) -> (StatusCode, String) {
    if !state.wrapped_server.running() {
        state.wrapped_server.wait_for_reload().await;
    }

    (StatusCode::OK, "OK".into())
}

#[tokio::main]
async fn main() {
    logging::init_tracing();
    let cli = Cli::parse();

    let wrapped_server = subproc::WrappedServer::new(cli.command, cli.port);
    let state = Arc::new(ServerState {
        wrapped_server,
        subprocess_port: cli.subprocess_port,
    });

    let frag_routes = Router::new()
        .route("/status", get(status))
        .route("/restart", post(restart))
        .route("/interrupt", post(interrupt))
        .route("/await", get(wait_for_reload))
        .route("/upload/{*filename}", post(upload));

    let app = Router::new()
        .nest("/_frag", frag_routes)
        .fallback(proxy_request)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("Server starting on http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    // Set up signal handlers
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to set up SIGTERM handler");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down");
        }
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM, shutting down");
        }
        _ = server_handle => {
            tracing::info!("Server terminated unexpectedly");
        }
    }

    // Ensure clean shutdown
    tracing::info!("Shutting down");
}
