
use sqlx::PgPool;

// #[tracing::instument(skip(_pool))]
pub async fn generate(_pool: &PgPool, _run_id: i32) -> Result<(), anyhow::Error> {
    Ok(())
}