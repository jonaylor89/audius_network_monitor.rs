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

    // Index data from the discovery node postgres DB
    // into the separate network monitoring postgres DB
    let run_id = discovery::index(&pool).await?;

    // Fetch data (CIDs and Users) from content nodes
    // and save it into the network monitoring postgres DB
    content::index(&pool, run_id, configuration.content).await?;

    // Run OLAP-type queries on the network monitoring DB
    // and export the data to the prometheus push-gateway
    // to be later scraped by prometheus
    metrics::generate(&pool, run_id, configuration.metrics).await?;

    Ok(())
}
