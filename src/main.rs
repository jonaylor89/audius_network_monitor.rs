use audius_network_monitor::{
    configuration::get_configuration,
    db::{create_foreign_connection, get_connection_pool},
    discovery::index_discovery,
    telemetry::{get_subscriber, init_subscriber}, content::index_content,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "audius_network_monitoring".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    // Setup DB
    let pool = get_connection_pool(&configuration.database);
    sqlx::migrate!("./migrations").run(&pool).await?;
    // create_foreign_connection(&pool, &configuration.foreign_database).await?;

    // Index Discovery
    let run_id = 1;
    // let run_id = index_discovery(pool).await?;

    // Index Content
    index_content(pool, run_id).await?;

    // Generate Metrics

    Ok(())
}
