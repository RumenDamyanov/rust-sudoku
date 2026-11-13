//! sudoku-server — REST API for sudoku generation and solving.

use std::net::SocketAddr;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let app = rumenx_sudoku::server::create_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
