use audius_network_monitor::{
    configuration::get_configuration,
    content::index_content,
    db::{get_connection_pool, create_foreign_connection},
    telemetry::{get_subscriber, init_subscriber}, discovery::index_discovery,
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

    // Index Discovery
    let run_id = index_discovery(&pool).await?;
    tracing::info!("{}", run_id);

    // Index Content
    index_content(pool, run_id).await?;

    // Generate Metrics

    Ok(())
}
