use futures::{future::join_all, stream::futures_unordered::FuturesUnordered};
use sqlx::PgPool;
use thiserror::Error;
use tokio::join;
use color_eyre::eyre::Result;

use crate::{
    configuration::ContentSettings,
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
pub async fn index(pool: &PgPool, run_id: i32, config: ContentSettings) -> Result<()> {
    let content_nodes = get_content_nodes(pool, run_id).await?;

    let tasks = content_nodes
        .into_iter()
        .filter_map(|cnode| {
            if config.deregistered_nodes.contains(&cnode.endpoint) {
                tracing::info!("skipping {} because it is deregistered", cnode.endpoint);
                return None;
            }

            let pool_clone = pool.clone();
            Some(tokio::spawn(async move {
                check_users(pool_clone, run_id, cnode).await
            }))
        })
        .collect::<FuturesUnordered<_>>();

    join_all(tasks).await;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_content_nodes(pool: &PgPool, run_id: i32) -> Result<Vec<ContentNode>> {
    let content_nodes = sqlx::query!(
        r#"
        SELECT spid, endpoint
        FROM network_monitoring_content_nodes
        WHERE run_id = $1; 
        "#,
        run_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| ContentNode {
        endpoint: row.endpoint,
        spid: row.spid,
    })
    .collect::<Vec<ContentNode>>();

    Ok(content_nodes)
}

#[tracing::instrument(skip(pool))]
async fn check_users(pool: PgPool, run_id: i32, cnode: ContentNode) -> Result<()> {
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
) -> Result<()> {
    for offset in (0..count).step_by(BATCH_SIZE) {
        let wallet_batch = get_batch(replica, pool, run_id, spid, offset).await?;
        if wallet_batch.is_empty() {
            continue;
        }

        let clock_values = get_user_clock_values(endpoint, wallet_batch).await?;

        save_batch(replica, pool, run_id, spid, &clock_values).await?;
    }

    Ok(())
}

#[tracing::instrument(skip(wallet_batch))]
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
) -> Result<Vec<String>> {
    let batch = match replica {
        Replica::Primary => sqlx::query!(
            r#"
                    SELECT wallet 
                    FROM network_monitoring_users
                    WHERE run_id = $1
                    AND primaryspid = $2
                    ORDER BY user_id 
                    OFFSET $3
                    LIMIT $4; 
                "#,
            run_id,
            spid,
            offset,
            BATCH_SIZE as i64,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.wallet)
        .collect::<Vec<String>>(),
        Replica::Secondary1 => sqlx::query!(
            r#"
                    SELECT wallet 
                    FROM network_monitoring_users
                    WHERE run_id = $1
                    AND secondary1spid = $2
                    ORDER BY user_id 
                    OFFSET $3
                    LIMIT $4; 
                "#,
            run_id,
            spid,
            offset,
            BATCH_SIZE as i64,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.wallet)
        .collect::<Vec<String>>(),
        Replica::Secondary2 => sqlx::query!(
            r#"
                    SELECT wallet 
                    FROM network_monitoring_users
                    WHERE run_id = $1
                    AND secondary2spid = $2
                    ORDER BY user_id 
                    OFFSET $3
                    LIMIT $4; 
                "#,
            run_id,
            spid,
            offset,
            BATCH_SIZE as i64,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.wallet)
        .collect::<Vec<String>>(),
    };

    Ok(batch)
}

#[tracing::instrument(skip(pool, clock_values))]
async fn save_batch(
    replica: &Replica,
    pool: &PgPool,
    run_id: i32,
    spid: i32,
    clock_values: &Vec<WalletClockPair>,
) -> Result<()> {
    let mini_batch_size = 500;
    let count = clock_values.len();

    for offset in (0..count).step_by(mini_batch_size) {
        let end = if (offset + mini_batch_size) >= count {
            count - 1
        } else {
            offset + mini_batch_size
        };

        let mini_batch = &clock_values[offset..end];

        let (wallets, clocks): (Vec<String>, Vec<i32>) = mini_batch
            .iter()
            .map(|pair| (pair.wallet_public_key.clone(), pair.clock))
            .unzip();

        match replica {
            Replica::Primary => {
                sqlx::query!(
                    r#"
                    UPDATE network_monitoring_users as nm_users
                    SET primary_clock_value = tmp.clock
                    FROM UNNEST($2::text[], $3::int[]) AS tmp(wallet, clock)
                    WHERE nm_users.wallet = tmp.wallet
                    AND nm_users.run_id = $1;
                "#,
                    run_id,
                    &wallets,
                    &clocks,
                )
            }
            Replica::Secondary1 => {
                sqlx::query!(
                    r#"
                    UPDATE network_monitoring_users as nm_users
                    SET secondary1_clock_value = tmp.clock
                    FROM UNNEST($2::text[], $3::int[]) AS tmp(wallet, clock)
                    WHERE nm_users.wallet = tmp.wallet
                    AND nm_users.run_id = $1;
                "#,
                    run_id,
                    &wallets,
                    &clocks,
                )
            }
            Replica::Secondary2 => {
                sqlx::query!(
                    r#"
                    UPDATE network_monitoring_users as nm_users
                    SET secondary2_clock_value = tmp.clock
                    FROM UNNEST($2::text[], $3::int[]) AS tmp(wallet, clock)
                    WHERE nm_users.wallet = tmp.wallet
                    AND nm_users.run_id = $1;
                "#,
                    run_id,
                    &wallets,
                    &clocks,
                )
            }
        }
        .execute(pool)
        .await?;
    }

    Ok(())
}
