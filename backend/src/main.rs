use axum::http::HeaderValue;
use clap::Parser;
use gold_price_backend::{
    app::build_app,
    config::{load_database_url, ReleasePaths},
    repository::gold_prices::GoldPriceRepository,
};
use sea_orm::Database;
use std::{error::Error, io, net::SocketAddr};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(clap::Parser)]
struct Cli {
    #[arg(long)]
    host: std::net::IpAddr,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let executable = std::env::current_exe()?;
    let release_dir = executable.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "executable does not have a parent directory",
        )
    })?;
    let paths = ReleasePaths::from_release_dir(release_dir);

    std::fs::create_dir_all(&paths.logs_dir)?;
    let log_file = paths.logs_dir.join("gold-price.log");
    let file_appender = tracing_appender::rolling::never(&paths.logs_dir, "gold-price.log");
    let (file_writer, _worker_guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(file_writer))
        .init();

    let database_url = load_database_url(&paths)?;
    let database = Database::connect(database_url).await?;
    let app = build_app(
        GoldPriceRepository::new(database),
        paths.dist_dir.clone(),
        HeaderValue::from_static("http://localhost:5173"),
    );

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(cli.host, cli.port)).await?;
    let address = listener.local_addr()?;
    info!(
        %address,
        dist = %paths.dist_dir.display(),
        log_file = %log_file.display(),
        "gold price service started"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
