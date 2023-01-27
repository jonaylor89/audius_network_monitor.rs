use audius_network_monitor::{
    configuration, content,
    db::{create_foreign_connection, get_connection_pool},
    discovery, metrics,
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

    let configuration = configuration::read().expect("Failed to read configuration");

    // Setup DB
    let pool = get_connection_pool(&configuration.database);
    sqlx::migrate!("./migrations").run(&pool).await?;
    create_foreign_connection(&pool, &configuration.foreign_database).await?;

    // Index Discovery
    let run_id = discovery::index(&pool).await?;

    // Index Content
    content::index(&pool, run_id).await?;

    metrics::generate(&pool, run_id).await?;

    Ok(())
}
