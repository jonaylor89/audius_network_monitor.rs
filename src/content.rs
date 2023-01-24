use futures::{future::join_all, stream::futures_unordered::FuturesUnordered};
use sqlx::PgPool;
use tokio::join;

use crate::domain::ContentNode;

const BATCH_SIZE: usize = 5_000;

#[tracing::instrument(skip(pool))]
pub async fn index_content(pool: PgPool, run_id: i32) -> Result<(), anyhow::Error> {
    let content_nodes = vec![ContentNode::default()];

    let tasks = content_nodes
        .into_iter()
        .map(|cnode| {
            let pool_clone = pool.clone();
            tokio::spawn(async move { check_users(pool_clone, run_id, cnode).await })
        })
        .collect::<FuturesUnordered<_>>();

    join_all(tasks).await;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn check_users(pool: PgPool, run_id: i32, cnode: ContentNode) -> Result<(), anyhow::Error> {
    let ContentNode { spid, endpoint } = cnode;
    let (primary_count, secondary1_count, secondary2_count) =
        get_user_counts(&pool, run_id, spid).await?;

    let (primary_result, secondary1_result, secondary2_result) = join!(
        check_primaries(&pool, run_id, primary_count, spid, &endpoint),
        check_secondaries1(&pool, run_id, secondary1_count, spid, &endpoint),
        check_secondaries2(&pool, run_id, secondary2_count, spid, &endpoint),
    );

    primary_result?;
    secondary1_result?;
    secondary2_result?;

    Ok(())
}

#[tracing::instrument(skip(_pool))]
async fn get_user_counts(
    _pool: &PgPool,
    run_id: i32,
    spid: i32,
) -> Result<(i32, i32, i32), anyhow::Error> {
    Ok((0, 0, 0))
}

#[tracing::instrument(skip(pool))]
async fn check_primaries(
    pool: &PgPool,
    run_id: i32,
    count: i32,
    spid: i32,
    endpoint: &str,
) -> Result<(), anyhow::Error> {
    for offset in (0..count).step_by(BATCH_SIZE) {
        let wallet_batch = get_primary_batch(pool, run_id, spid, offset).await?;
        if wallet_batch.is_empty() {
            continue;
        }

        let clock_values = get_user_clock_values(endpoint, wallet_batch).await?;

        save_primary_batch(pool, run_id, spid, clock_values).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn check_secondaries1(
    pool: &PgPool,
    run_id: i32,
    count: i32,
    spid: i32,
    endpoint: &str,
) -> Result<(), anyhow::Error> {
    for offset in (0..count).step_by(BATCH_SIZE) {
        let wallet_batch = get_secondary1_batch(pool, run_id, spid, offset).await?;
        if wallet_batch.is_empty() {
            continue;
        }

        let clock_values = get_user_clock_values(endpoint, wallet_batch).await?;

        save_secondary1_batch(pool, run_id, spid, clock_values).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn check_secondaries2(
    pool: &PgPool,
    run_id: i32,
    count: i32,
    spid: i32,
    endpoint: &str,
) -> Result<(), anyhow::Error> {
    for offset in (0..count).step_by(BATCH_SIZE) {
        let wallet_batch = get_secondary2_batch(pool, run_id, spid, offset).await?;
        if wallet_batch.is_empty() {
            continue;
        }

        let clock_values = get_user_clock_values(endpoint, wallet_batch).await?;

        save_secondary2_batch(pool, run_id, spid, clock_values).await?;
    }

    Ok(())
}

#[tracing::instrument]
async fn get_user_clock_values(
    endpoint: &str,
    wallet_batch: Vec<String>,
) -> Result<Vec<(String, i32)>, anyhow::Error> {
    Ok(vec![("whoa".to_string(), 1)])
}

#[tracing::instrument(skip(pool))]
async fn get_primary_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    offset: i32,
) -> Result<Vec<String>, anyhow::Error> {
    Ok(vec!["whoa".to_string()])
}

#[tracing::instrument(skip(pool))]
async fn get_secondary1_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    offset: i32,
) -> Result<Vec<String>, anyhow::Error> {
    Ok(vec!["whoa".to_string()])
}

#[tracing::instrument(skip(pool))]
async fn get_secondary2_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    offset: i32,
) -> Result<Vec<String>, anyhow::Error> {
    Ok(vec!["whoa".to_string()])
}

#[tracing::instrument(skip(pool))]
async fn save_primary_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    clock_values: Vec<(String, i32)>,
) -> Result<(), anyhow::Error> {
    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn save_secondary1_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    clock_values: Vec<(String, i32)>,
) -> Result<(), anyhow::Error> {
    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn save_secondary2_batch(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    clock_values: Vec<(String, i32)>,
) -> Result<(), anyhow::Error> {
    Ok(())
}
