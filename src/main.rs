use axum::{routing::{get, post}, Router};
use routes::{send_notification_handler, sse_handler, ws_handler};
use tokio::sync::broadcast;
use tower_http::{services::{ServeDir, ServeFile}, trace::{DefaultMakeSpan, TraceLayer}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use types::Channels;
use core::net::SocketAddr;

mod routes;
mod types;
mod websocket;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Environment variables to load");

    tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "level=debug,tower_http=debug".into())
    )
    .with(
        tracing_subscriber::fmt::layer()
    ).init();

    let port: u16 = std::env::var("PORT").unwrap_or("3000".to_string()).parse().unwrap();
    serve(create_router(), port).await
}


fn create_router() -> Router {
    let (tx, _rx) = broadcast::channel(100);
    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
    Router::new()
        .fallback_service(serve_dir)
        .route("/ws/:user_id", get(ws_handler))
        .route("/sse/:user_id", get(sse_handler))
        .route("/admin/send_notification/:user_id", post(send_notification_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default())
        )
        .with_state(Channels {tx})
        

}

async fn serve(app: Router, port: u16) {
    let address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    
    axum::serve(
        listener, 
        app
        .into_make_service_with_connect_info::<SocketAddr>())
    .await
    .unwrap()
}
