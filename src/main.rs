use audius_network_monitor::{
    configuration::{get_configuration, self},
    content::index_content,
    db::{create_foreign_connection, get_connection_pool},
    discovery::index_discovery,
    metrics::generate_metrics,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "audius_network_monitor".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    // Setup DB
    let pool = get_connection_pool(&configuration.database);
    sqlx::migrate!("./migrations").run(&pool).await?;
    create_foreign_connection(&pool, &configuration.foreign_database).await?;

    let run_id = index_discovery(&pool).await?;

    index_content(&pool, run_id).await?;

    generate_metrics(&pool, run_id).await?;

    Ok(())
}
