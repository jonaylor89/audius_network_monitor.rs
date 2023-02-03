use audius_network_monitor::{
    configuration, content,
    db::{create_foreign_connection, get_connection_pool},
    discovery, metrics,
    telemetry::{get_subscriber, init_subscriber},
};
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let subscriber = get_subscriber(
        "audius_network_monitor".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = configuration::read().expect("Failed to read configuration");

    let pool = get_connection_pool(&configuration.database);
    sqlx::migrate!("./migrations").run(&pool).await?;
    create_foreign_connection(&pool, &configuration.foreign_database).await?;

    let run_id = discovery::index(&pool).await?;

    content::index(&pool, run_id, configuration.content).await?;

    metrics::generate(&pool, run_id, configuration.metrics).await?;

    Ok(())
}
