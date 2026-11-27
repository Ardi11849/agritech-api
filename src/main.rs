use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agritech_api::config::AppConfig;
use agritech_api::{build_app_state, create_app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,agritech_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Starting agritech-api on port {}", config.port);

    let connect_options = config.pg_connect_options()?;
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect_with(connect_options)
        .await?;

    tracing::info!("Connected to PostgreSQL successfully");

    // 1. Auto-migration
    tracing::info!("Running database auto-migrations...");
    agritech_api::infra::migrator::run_migrations(&pool).await?;
    tracing::info!("Auto-migrations completed successfully");

    // 2. Clean Legacy Dummy Data & Run Idempotent Database Seeder
    tracing::info!("Checking and running idempotent database seeder...");
    agritech_api::infra::seeder::seed_database(&pool).await?;
    tracing::info!("Database seeder check completed successfully");

    // 3. Mount App and Listener
    let state = build_app_state(pool, config.clone());
    let app = create_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on http://{}", addr);

    if config.enable_swagger {
        tracing::info!(
            "Swagger UI documentation available at http://{}/swagger-ui",
            addr
        );
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
