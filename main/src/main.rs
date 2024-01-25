use server::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "foundry-forum=trace,server=trace,axum_login=debug,tower_sessions=debug,sqlx=warn".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .unwrap();

    let app = Server::new("127.0.0.1:3000").await.unwrap();

    let delete_task = app.get_delete_task();
    let server_task = app.serve();
    let _ = tokio::join!(delete_task, server_task);
}
