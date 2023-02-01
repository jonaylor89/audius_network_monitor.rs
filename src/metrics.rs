use prometheus::labels;
use sqlx::{
    types::chrono::{DateTime, Utc},
    PgPool,
};

use crate::{configuration::MetricsSettings, prometheus::USER_COUNT_GAUGE};

#[tracing::instrument(skip(pool))]
pub async fn generate(pool: &PgPool, run_id: i32, config: MetricsSettings) -> anyhow::Result<()> {
    // GENERATE METRICS
    let run_time_start = get_run_time_start(pool, run_id).await?;
    let user_count = get_user_count(pool, run_id).await?;
    let all_user_count = get_all_user_count(pool, run_id).await?;
    let primary_user_count = get_primary_user_count(pool, run_id).await?;
    let fully_synced_users_count = get_fully_synced_users_count(pool, run_id).await?;
    let partially_synced_users_count = get_partially_synced_users_count(pool, run_id).await?;
    let unsynced_users_count = get_unsynced_users_count(pool, run_id).await?;
    let users_with_null_primary_clock = get_users_with_null_primary_clock(pool, run_id).await?;
    let users_with_unhealthy_replica = get_users_with_unhealthy_replica(pool, run_id).await?;
    let users_with_all_foundation_node_replica_set =
        get_users_with_all_foundation_node_replica_set(pool, run_id, &config.foundation_nodes)
            .await?;

    let users_with_no_foundation_node_replica_set_count =
        get_users_with_entire_replica_set_not_in_spid_set_count(
            pool,
            run_id,
            &config.foundation_nodes,
        )
        .await?;

    let users_status_by_primary = get_users_status_by_primary(pool, run_id).await?;
    let users_status_by_replica = get_users_status_by_replica(pool, run_id).await?;

    // REGISTER METRICS
    USER_COUNT_GAUGE
        .with_label_values(&["run_id"])
        .set(user_count);

    let metric_families = prometheus::gather();
    prometheus::push_metrics(
        "network-monitoring",
        labels! {"run_id".to_owned() => run_id.to_string(),},
        &config.push_gateway,
        metric_families,
        Some(prometheus::BasicAuthentication {
            username: "user".to_owned(),
            password: "pass".to_owned(),
        }),
    )?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_run_time_start(pool: &PgPool, run_id: i32) -> anyhow::Result<DateTime<Utc>> {
    Ok(Utc::now())
}

#[tracing::instrument(skip(pool))]
async fn get_user_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_all_user_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_primary_user_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_fully_synced_users_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_partially_synced_users_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_unsynced_users_count(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_null_primary_clock(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_unhealthy_replica(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_all_foundation_node_replica_set(
    pool: &PgPool,
    run_id: i32,
    foundation_nodes: &Vec<u16>,
) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_entire_replica_set_not_in_spid_set_count(
    pool: &PgPool,
    run_id: i32,
    spids: &Vec<u16>,
) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_status_by_primary(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}

#[tracing::instrument(skip(pool))]
async fn get_users_status_by_replica(pool: &PgPool, run_id: i32) -> anyhow::Result<i64> {
    Ok(0)
}
