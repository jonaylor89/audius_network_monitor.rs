
use sqlx::PgPool;

// #[tracing::instument(skip(_pool))]
pub async fn generate_metrics(_pool: &PgPool, _run_id: i32) -> Result<(), anyhow::Error> {
    Ok(())
}