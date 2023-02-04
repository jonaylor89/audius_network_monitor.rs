use color_eyre::eyre::Result;
use num_traits::cast::ToPrimitive;
use prometheus::labels;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        BigDecimal,
    },
    PgPool,
};

use crate::{
    configuration::MetricsSettings,
    prometheus::{
        ALL_USER_COUNT_GAUGE, FULLY_SYNCED_USERS_COUNT_GAUGE,
        FULLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE, FULLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE,
        NULL_PRIMARY_USERS_COUNT_GAUGE, PARTIALLY_SYNCED_USERS_COUNT_GAUGE,
        PARTIALLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE, PARTIALLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE,
        PRIMARY_USER_COUNT_GAUGE, UNHEALTHY_REPLICA_USERS_COUNT_GAUGE, UNSYNCED_USERS_COUNT_GAUGE,
        UNSYNCED_USER_BY_PRIMARY_COUNT_GAUGE, UNSYNCED_USER_BY_REPLICA_COUNT_GAUGE,
        USERS_WITH_ALL_FOUNDATION_NODE_REPLICA_SET_GAUGE,
        USERS_WITH_NO_FOUNDATION_NODE_REPLICA_SET_GAUGE, USER_COUNT_GAUGE, TOTAL_JOB_DURATION_GAUGE,
    },
};

pub struct CNodeCount {
    pub endpoint: String,
    pub count: i64,
}

pub struct CNodeSyncedStatus {
    pub spid: i32,
    pub endpoint: String,
    pub fully_synced_count: i64,
    pub partially_synced_count: i64,
    pub unsynced_count: i64,
}

#[tracing::instrument(skip(pool))]
pub async fn generate(pool: &PgPool, run_id: i32, config: MetricsSettings) -> Result<()> {
    // GENERATE METRICS
    let run_time_start = get_run_start_time(pool, run_id).await?;
    let user_count = get_user_count(pool, run_id).await?;
    let all_user_count = get_all_user_count(pool, run_id).await?;
    let primary_user_count = get_primary_user_count(pool, run_id).await?;
    let fully_synced_users_count = get_fully_synced_users_count(pool, run_id).await?;
    let partially_synced_users_count = get_partially_synced_users_count(pool, run_id).await?;
    let unsynced_users_count = get_unsynced_users_count(pool, run_id).await?;
    let users_with_null_primary_clock = get_users_with_null_primary_clock(pool, run_id).await?;
    let users_with_unhealthy_replica = get_users_with_unhealthy_replica(pool, run_id).await?;
    let users_with_all_foundation_node_replica_set =
        get_users_with_entire_replica_in_spid_set_count(pool, run_id, &config.foundation_nodes)
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
        .with_label_values(&[&run_id.to_string()])
        .set(user_count);

    for cnode_count in all_user_count {
        ALL_USER_COUNT_GAUGE
            .with_label_values(&[&cnode_count.endpoint, &run_id.to_string()])
            .set(cnode_count.count);
    }

    for cnode_count in primary_user_count {
        PRIMARY_USER_COUNT_GAUGE
            .with_label_values(&[&cnode_count.endpoint, &run_id.to_string()])
            .set(cnode_count.count);
    }

    FULLY_SYNCED_USERS_COUNT_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(fully_synced_users_count);

    PARTIALLY_SYNCED_USERS_COUNT_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(partially_synced_users_count);

    UNSYNCED_USERS_COUNT_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(unsynced_users_count);

    NULL_PRIMARY_USERS_COUNT_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(users_with_null_primary_clock);

    UNHEALTHY_REPLICA_USERS_COUNT_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(users_with_unhealthy_replica);

    USERS_WITH_ALL_FOUNDATION_NODE_REPLICA_SET_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(users_with_all_foundation_node_replica_set);

    USERS_WITH_NO_FOUNDATION_NODE_REPLICA_SET_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(users_with_no_foundation_node_replica_set_count);

    for users_status in users_status_by_primary {
        FULLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.fully_synced_count);
        PARTIALLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.partially_synced_count);
        UNSYNCED_USER_BY_PRIMARY_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.unsynced_count);
    }

    for users_status in users_status_by_replica {
        FULLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.fully_synced_count);
        PARTIALLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.partially_synced_count);
        UNSYNCED_USER_BY_REPLICA_COUNT_GAUGE
            .with_label_values(&[&users_status.endpoint, &run_id.to_string()])
            .set(users_status.unsynced_count);
    }

    let total_run_time = Utc::now() - run_time_start;
    TOTAL_JOB_DURATION_GAUGE
        .with_label_values(&[&run_id.to_string()])
        .set(total_run_time.num_seconds());

    let metric_families = prometheus::gather();
    prometheus::push_metrics(
        "network-monitoring",
        labels! {"run_id".to_owned() => run_id.to_string(),},
        &config.push_gateway,
        metric_families,
        None,
    )?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn get_run_start_time(pool: &PgPool, run_id: i32) -> Result<DateTime<Utc>> {
    let run_start_time = sqlx::query!(
        r#"
    SELECT created_at
    FROM 
        network_monitoring_index_blocks
    WHERE
        run_id = $1
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .created_at;

    Ok(run_start_time)
}

#[tracing::instrument(skip(pool))]
async fn get_user_count(pool: &PgPool, run_id: i32) -> Result<i64> {
    let user_count = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE run_id = $1
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(user_count)
}

#[tracing::instrument(skip(pool))]
async fn get_all_user_count(pool: &PgPool, run_id: i32) -> Result<Vec<CNodeCount>> {
    let all_user_count: Vec<CNodeCount> = sqlx::query!(
        r#"
        SELECT joined.endpoint AS endpoint, COUNT(*) AS count
        FROM (
            (SELECT * FROM network_monitoring_content_nodes WHERE run_id = $1) AS cnodes
        JOIN
        (
            SELECT user_id, unnest(string_to_array(replica_set, ',')) AS user_endpoint 
            FROM network_monitoring_users
            WHERE run_id = $1
        ) as unnested_users
        ON
            cnodes.endpoint = unnested_users.user_endpoint
        ) AS joined
        GROUP BY 
            joined.endpoint; 
    "#,
        run_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| CNodeCount {
        endpoint: row.endpoint,
        count: row.count.unwrap_or(0),
    })
    .collect::<Vec<CNodeCount>>();

    Ok(all_user_count)
}

#[tracing::instrument(skip(pool))]
async fn get_primary_user_count(pool: &PgPool, run_id: i32) -> Result<Vec<CNodeCount>> {
    let primary_user_count: Vec<CNodeCount> = sqlx::query!(
        r#"
            SELECT 
            joined.endpoint AS endpoint, COUNT(*) AS count
        FROM (
            (SELECT * FROM network_monitoring_users WHERE run_id = $1) AS current_users
        JOIN
            (SELECT * FROM network_monitoring_content_nodes WHERE run_id = $1) AS cnodes
        ON
            current_users.primaryspid = cnodes.spid
        ) AS joined
        GROUP BY 
            joined.endpoint 
    "#,
        run_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| CNodeCount {
        endpoint: row.endpoint,
        count: row.count.unwrap_or(0),
    })
    .collect::<Vec<CNodeCount>>();

    Ok(primary_user_count)
}

#[tracing::instrument(skip(pool))]
async fn get_fully_synced_users_count(pool: &PgPool, run_id: i32) -> Result<i64> {
    let fully_synced_users_count = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE
        run_id = $1
    AND 
        primary_clock_value IS NOT NULL
    AND 
        primary_clock_value != -2
    AND
        primary_clock_value = secondary1_clock_value
    AND
        secondary1_clock_value = secondary2_clock_value; 
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(fully_synced_users_count)
}

#[tracing::instrument(skip(pool))]
async fn get_partially_synced_users_count(pool: &PgPool, run_id: i32) -> Result<i64> {
    let partially_synced_users_count = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE 
        run_id = $1
    AND 
        primary_clock_value IS NOT NULL
    AND 
        primary_clock_value != -2
    AND ( 
        primary_clock_value = secondary1_clock_value
        OR
        primary_clock_value = secondary2_clock_value
    )
    AND 
        secondary1_clock_value != secondary2_clock_value; 
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(partially_synced_users_count)
}

#[tracing::instrument(skip(pool))]
async fn get_unsynced_users_count(pool: &PgPool, run_id: i32) -> Result<i64> {
    let unsynced_users_count = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE 
        run_id = $1
    AND 
        primary_clock_value IS NOT NULL
    AND 
        primary_clock_value != -2
    AND 
        primary_clock_value != secondary1_clock_value
    AND
        primary_clock_value != secondary2_clock_value; 
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(unsynced_users_count)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_null_primary_clock(pool: &PgPool, run_id: i32) -> Result<i64> {
    let users_with_null_primary_clock = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE 
        run_id = $1
    AND 
        primary_clock_value IS NULL; 
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(users_with_null_primary_clock)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_unhealthy_replica(pool: &PgPool, run_id: i32) -> Result<i64> {
    let users_with_unhealthy_replica = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE 
        run_id = $1
    AND (
        primary_clock_value = -2
        OR
        secondary1_clock_value = -2
        OR 
        secondary2_clock_value = -2
    );
    "#,
        run_id
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(users_with_unhealthy_replica)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_entire_replica_in_spid_set_count(
    pool: &PgPool,
    run_id: i32,
    foundation_nodes: &Vec<i32>,
) -> Result<i64> {
    let users_with_all_foundation_node_replica_set = sqlx::query!(
        r#"
    SELECT COUNT(*) as user_count
    FROM network_monitoring_users
    WHERE
        run_id = $1
    AND 
        primaryspid = ANY( $2 )
    AND
        secondary1spid = ANY( $2 )
    AND 
        secondary2spid = ANY( $2 ); 
    "#,
        run_id,
        foundation_nodes.as_slice()
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(users_with_all_foundation_node_replica_set)
}

#[tracing::instrument(skip(pool))]
async fn get_users_with_entire_replica_set_not_in_spid_set_count(
    pool: &PgPool,
    run_id: i32,
    spids: &Vec<i32>,
) -> Result<i64> {
    let users_with_entire_replica_set_not_in_spid_set = sqlx::query!(
        r#"
        SELECT COUNT(*) as user_count
        FROM network_monitoring_users
        WHERE
            run_id = $1
        AND 
            primaryspid != ALL( $2 )
        AND
            secondary1spid != ALL( $2 )
        AND 
            secondary2spid != ALL( $2 );
   "#,
        run_id,
        spids,
    )
    .fetch_one(pool)
    .await?
    .user_count
    .unwrap_or(0);

    Ok(users_with_entire_replica_set_not_in_spid_set)
}

#[tracing::instrument(skip(pool))]
async fn get_users_status_by_primary(pool: &PgPool, run_id: i32) -> Result<Vec<CNodeSyncedStatus>> {
    let users_status_by_primary = sqlx::query!(r#"
        SELECT fully_synced.spid, cnodes.endpoint, fully_synced.fully_synced_count, partially_synced.partially_synced_count, unsynced.unsynced_count
        FROM (
            SELECT primaryspid AS spid, COUNT(*) as fully_synced_count
            FROM network_monitoring_users
            WHERE
                run_id = $1
            AND 
                primary_clock_value IS NOT NULL
            AND
                primary_clock_value = secondary1_clock_value
            AND
                secondary1_clock_value = secondary2_clock_value
            GROUP BY primaryspid
        ) AS fully_synced
        JOIN (
            SELECT primaryspid AS SPID, COUNT(*) AS partially_synced_count
            FROM network_monitoring_users
            WHERE 
                run_id = $1
            AND 
                primary_clock_value IS NOT NULL
            AND ( 
                primary_clock_value = secondary1_clock_value
                OR
                primary_clock_value = secondary2_clock_value
            )
            AND 
                secondary1_clock_value != secondary2_clock_value
            GROUP BY primaryspid
        ) AS partially_synced
        ON fully_synced.spid = partially_synced.spid
        JOIN (
            SELECT primaryspid AS spid, COUNT(*) AS unsynced_count
            FROM network_monitoring_users
            WHERE 
                run_id = $1
            AND 
                primary_clock_value IS NOT NULL
            AND 
                primary_clock_value != secondary1_clock_value
            AND
                primary_clock_value != secondary2_clock_value
            GROUP BY primaryspid
        ) AS unsynced
        ON fully_synced.spid = unsynced.spid
        JOIN (
            SELECT spid, endpoint
            FROM network_monitoring_content_nodes
            WHERE
                run_id = $1
        ) AS cnodes
        ON cnodes.spid = fully_synced.spid
        ORDER BY fully_synced.spid; 
    "#,
        run_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| CNodeSyncedStatus {
        spid: row.spid,
        endpoint: row.endpoint,
        fully_synced_count: row.fully_synced_count.unwrap_or(0),
        partially_synced_count: row.partially_synced_count.unwrap_or(0),
        unsynced_count: row.unsynced_count.unwrap_or(0),
    })
    .collect::<Vec<CNodeSyncedStatus>>();

    Ok(users_status_by_primary)
}

#[tracing::instrument(skip(pool))]
async fn get_users_status_by_replica(pool: &PgPool, run_id: i32) -> Result<Vec<CNodeSyncedStatus>> {
    let users_status_by_replica = sqlx::query!(
        r#"
        SELECT 
            fully_synced.spid, 
            cnodes.endpoint, 
            fully_synced.fully_synced_count, 
            partially_synced.partially_synced_count, 
            unsynced.unsynced_count
        FROM (
            SELECT 
                fully_synced_primary.spid AS spid, 
                (SUM(fully_synced_primary.fully_synced_count) +
                SUM(fully_synced_secondary1.fully_synced_count) +
                SUM(fully_synced_secondary2.fully_synced_count)) AS fully_synced_count
            FROM (
                SELECT primaryspid AS spid, COUNT(*) as fully_synced_count
                FROM network_monitoring_users
                WHERE
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND
                    primary_clock_value = secondary1_clock_value
                AND
                    secondary1_clock_value = secondary2_clock_value
                GROUP BY primaryspid
            ) AS fully_synced_primary
            JOIN (
                SELECT secondary1spid AS spid, COUNT(*) as fully_synced_count
                FROM network_monitoring_users
                WHERE
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND
                    primary_clock_value = secondary1_clock_value
                AND
                    secondary1_clock_value = secondary2_clock_value
                GROUP BY secondary1spid
            ) AS fully_synced_secondary1
            ON fully_synced_primary.spid = fully_synced_secondary1.spid
            JOIN (
                SELECT secondary2spid AS spid, COUNT(*) as fully_synced_count
                FROM network_monitoring_users
                WHERE
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND
                    primary_clock_value = secondary1_clock_value
                AND
                    secondary1_clock_value = secondary2_clock_value
                GROUP BY secondary2spid
            ) AS fully_synced_secondary2
            ON fully_synced_primary.spid = fully_synced_secondary2.spid
            GROUP BY fully_synced_primary.spid
        ) AS fully_synced
        JOIN (
            SELECT 
                partially_synced_primary.spid AS spid, 
                (SUM(partially_synced_primary.partially_synced_count) +
                SUM(partially_synced_secondary1.partially_synced_count) +
                SUM(partially_synced_secondary2.partially_synced_count)) AS partially_synced_count
            FROM (
                SELECT primaryspid AS SPID, COUNT(*) AS partially_synced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND ( 
                    primary_clock_value = secondary1_clock_value
                    OR
                    primary_clock_value = secondary2_clock_value
                )
                AND 
                    secondary1_clock_value != secondary2_clock_value
                GROUP BY primaryspid
            ) AS partially_synced_primary
            JOIN (
                SELECT secondary1spid AS SPID, COUNT(*) AS partially_synced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND ( 
                    primary_clock_value = secondary1_clock_value
                    OR
                    primary_clock_value = secondary2_clock_value
                )
                AND 
                    secondary1_clock_value != secondary2_clock_value
                GROUP BY secondary1spid
            ) AS partially_synced_secondary1
            ON partially_synced_primary.spid = partially_synced_secondary1.spid
            JOIN (
                SELECT secondary2spid AS SPID, COUNT(*) AS partially_synced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND ( 
                    primary_clock_value = secondary1_clock_value
                    OR
                    primary_clock_value = secondary2_clock_value
                )
                AND 
                    secondary1_clock_value != secondary2_clock_value
                GROUP BY secondary2spid
            ) AS partially_synced_secondary2
            ON partially_synced_primary.spid = partially_synced_secondary2.spid
            GROUP BY partially_synced_primary.spid
        ) AS partially_synced
        ON fully_synced.spid = partially_synced.spid
        JOIN (
            SELECT 
                unsynced_primary.spid AS spid, 
                (SUM(unsynced_primary.unsynced_count) +
                SUM(unsynced_secondary1.unsynced_count) +
                SUM(unsynced_secondary2.unsynced_count)) AS unsynced_count
            FROM (
                SELECT primaryspid AS spid, COUNT(*) AS unsynced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND 
                    primary_clock_value != secondary1_clock_value
                AND
                    primary_clock_value != secondary2_clock_value
                GROUP BY primaryspid
            ) AS unsynced_primary
            JOIN (
                SELECT secondary1spid AS spid, COUNT(*) AS unsynced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND 
                    primary_clock_value != secondary1_clock_value
                AND
                    primary_clock_value != secondary2_clock_value
                GROUP BY secondary1spid
            ) AS unsynced_secondary1
            ON unsynced_primary.spid = unsynced_secondary1.spid
            JOIN (
                SELECT secondary2spid AS spid, COUNT(*) AS unsynced_count
                FROM network_monitoring_users
                WHERE 
                    run_id = $1
                AND 
                    primary_clock_value IS NOT NULL
                AND 
                    primary_clock_value != secondary1_clock_value
                AND
                    primary_clock_value != secondary2_clock_value
                GROUP BY secondary2spid
            ) AS unsynced_secondary2
            ON unsynced_primary.spid = unsynced_secondary2.spid
            GROUP BY unsynced_primary.spid
        ) AS unsynced
        ON fully_synced.spid = unsynced.spid
        JOIN (
            SELECT spid, endpoint
            FROM network_monitoring_content_nodes
            WHERE
                run_id = $1 
        ) AS cnodes
        ON cnodes.spid = fully_synced.spid
        ORDER BY fully_synced.spid;
        "#,
        run_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| CNodeSyncedStatus {
        spid: row.spid,
        endpoint: row.endpoint,
        fully_synced_count: row
            .fully_synced_count
            .unwrap_or(BigDecimal::from(0))
            .to_i64()
            .unwrap_or(0),
        partially_synced_count: row
            .partially_synced_count
            .unwrap_or(BigDecimal::from(0))
            .to_i64()
            .unwrap_or(0),
        unsynced_count: row
            .unsynced_count
            .unwrap_or(BigDecimal::from(0))
            .to_i64()
            .unwrap_or(0),
    })
    .collect::<Vec<CNodeSyncedStatus>>();

    Ok(users_status_by_replica)
}
