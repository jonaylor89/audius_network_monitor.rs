use sqlx::PgPool;

use crate::configuration::MetricsSettings;

// #[tracing::instument(skip(_pool))]
pub async fn generate(
    _pool: &PgPool,
    _run_id: i32,
    _config: MetricsSettings,
) -> Result<(), anyhow::Error> {
    Ok(())
}
