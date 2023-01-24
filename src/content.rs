use futures::{future::join_all, stream::futures_unordered::FuturesUnordered};
use sqlx::PgPool;
use thiserror::Error;
use tokio::join;

use crate::{
    domain::{ContentNode, WalletClockPair},
    utils::{make_request, UserStatusPayload},
};

const BATCH_SIZE: usize = 5_000;

#[derive(Debug)]
enum Replica {
    Primary,
    Secondary1,
    Secondary2,
}

#[derive(Error, Debug)]
enum ContentNodeError {
    #[error("endpoint string is empty")]
    EndpointIsEmpty,
    #[error("request to content node failed")]
    ContentNodeRequestError,
    #[error("failed to query user counts")]
    UserCountsIsNone,
}

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
        check_replica(
            &Replica::Primary,
            &pool,
            run_id,
            primary_count,
            spid,
            &endpoint
        ),
        check_replica(
            &Replica::Secondary1,
            &pool,
            run_id,
            secondary1_count,
            spid,
            &endpoint
        ),
        check_replica(
            &Replica::Secondary2,
            &pool,
            run_id,
            secondary2_count,
            spid,
            &endpoint
        ),
    );

    primary_result?;
    secondary1_result?;
    secondary2_result?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_user_counts(
    pool: &PgPool,
    run_id: i32,
    spid: i32,
) -> Result<(i64, i64, i64), ContentNodeError> {
    let user_counts = sqlx::query!(
        r#"
        SELECT primary_group.spid as spid, primary_count, secondary1_count, secondary2_count
        FROM (
            (
                SELECT primaryspid AS spid, COUNT(*) AS primary_count
                FROM network_monitoring_users 
                WHERE run_id = $1
                GROUP BY primaryspid
            ) as primary_group
            JOIN 
            (
                SELECT secondary1spid AS spid, COUNT(*) AS secondary1_count 
                FROM network_monitoring_users 
                WHERE run_id = $1
                GROUP BY secondary1spid
            ) secondary1_group
            ON primary_group.spid = secondary1_group.spid
            JOIN
            (
                SELECT secondary2spid AS spid, COUNT(*) AS secondary2_count 
                FROM network_monitoring_users 
                WHERE run_id = $1
                GROUP BY secondary2spid
            ) secondary2_group
            ON primary_group.spid = secondary2_group.spid
        )
        WHERE primary_group.spid = $2; 
    "#,
        run_id,
        spid
    )
    .fetch_one(pool)
    .await
    .map_err(|_| ContentNodeError::UserCountsIsNone)?;

    Ok((
        user_counts
            .primary_count
            .ok_or(ContentNodeError::UserCountsIsNone)?,
        user_counts
            .secondary1_count
            .ok_or(ContentNodeError::UserCountsIsNone)?,
        user_counts
            .secondary2_count
            .ok_or(ContentNodeError::UserCountsIsNone)?,
    ))
}

#[tracing::instrument(skip(pool))]
async fn check_replica(
    replica: &Replica,
    pool: &PgPool,
    run_id: i32,
    count: i64,
    spid: i32,
    endpoint: &str,
) -> Result<(), anyhow::Error> {
    for offset in (0..count).step_by(BATCH_SIZE) {
        let wallet_batch = get_batch(replica, pool, run_id, spid, offset).await?;
        if wallet_batch.is_empty() {
            continue;
        }

        let clock_values = get_user_clock_values(endpoint, wallet_batch).await?;

        save_batch(replica, pool, run_id, spid, clock_values).await?;
    }

    Ok(())
}

#[tracing::instrument]
async fn get_user_clock_values(
    endpoint: &str,
    wallet_batch: Vec<String>,
) -> Result<Vec<WalletClockPair>, ContentNodeError> {
    if endpoint.is_empty() {
        return Err(ContentNodeError::EndpointIsEmpty);
    }

    let route = "/users/batch_clock_status";
    let url = format!("{endpoint}{route}");

    let payload = UserStatusPayload {
        wallet_public_keys: wallet_batch,
    };
    let results = make_request(&url, &payload)
        .await
        .map_err(|_| ContentNodeError::ContentNodeRequestError)?;

    Ok(results)
}

#[tracing::instrument(skip(pool))]
async fn get_batch(
    replica: &Replica,
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    offset: i64,
) -> Result<Vec<String>, anyhow::Error> {
    Ok(vec!["whoa".to_string()])
}

#[tracing::instrument(skip(pool))]
async fn save_batch(
    replica: &Replica,
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    clock_values: Vec<WalletClockPair>,
) -> Result<(), anyhow::Error> {
    Ok(())
}
