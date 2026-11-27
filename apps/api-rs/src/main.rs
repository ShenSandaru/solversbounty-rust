use solversbounty_api::config::load_from_env;
use solversbounty_api::http::routes;
use solversbounty_api::{build_state, AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&env_path);
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let config = load_from_env().unwrap_or_else(|err| {
        eprintln!("Configuration error: {err}");
        std::process::exit(1);
    });
    let addr = format!("0.0.0.0:{}", config.port);
    let state: AppState = build_state(config).await;
    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Failed to bind {addr}: {err}");
            std::process::exit(1);
        });
    tracing::info!("Listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("Server exited with error");
}