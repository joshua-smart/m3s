use anyhow::Result;
use args::Args;
use axum::Router;
use clap::Parser as _;
use tracing::info;

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        directory,
        log_level,
        address,
        port,
    } = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .compact()
        .init();

    let directory = directory
        .map(Ok)
        .unwrap_or_else(|| std::env::current_dir())?;
    info!("Starting at {directory:?}");

    let app = Router::new();

    let listener = tokio::net::TcpListener::bind((address.as_str(), port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
