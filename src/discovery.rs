use sqlx::PgPool;

#[tracing::instrument()]
pub async fn index_discovery(pool: PgPool) -> Result<i32, anyhow::Error> {
    // create new run
    let run_id = create_new_run(&pool).await?;

    delete_old_run_data(&pool, run_id).await?;

    import_users(&pool, run_id).await?;

    import_content_nodes(&pool, run_id).await?;

    import_cids(&pool, run_id).await?;

    Ok(run_id)
}

#[tracing::instrument(skip(pool))]
async fn create_new_run(pool: &PgPool) -> Result<i32, anyhow::Error> {
    // get latest block number
    let latest_block_number = 100_000;
    // let latest_block_number = sqlx::query!(
    //     r#"
    //     SELECT number FROM blocks WHERE is_current = TRUE LIMIT 1;
    //     "#,
    // )
    // .fetch_optional(pool)
    // .await?;

    // create new run in DB
    let run_id = sqlx::query!(
        r#"
        INSERT INTO network_monitoring_index_blocks (
            is_current, 
            blocknumber, 
            is_complete,
            created_at
        ) VALUES (
            TRUE,
            $1,
            FALSE,
            NOW()
        )
        RETURNING run_id; 
    "#,
        latest_block_number,
    )
    .fetch_one(pool)
    .await?
    .run_id;

    // remove is_current from latest run
    sqlx::query!(
        r#"
        UPDATE network_monitoring_index_blocks 
        SET is_current = FALSE
        WHERE blocknumber != $1;
    "#,
        latest_block_number,
    )
    .execute(pool)
    .await?;

    // return new run id
    Ok(run_id)
}

#[tracing::instrument(skip(pool))]
async fn delete_old_run_data(pool: &PgPool, run_id: i32) -> Result<(), anyhow::Error> {
    // Number of runs to keep in the DB
    let latest_runs_to_keep = 3;
    let to_delete = run_id - latest_runs_to_keep;

    if to_delete <= 0 {
        tracing::info!("no previous run data deleted");
        return Ok(());
    }

    sqlx::query!(
        r#"
        DELETE FROM network_monitoring_index_blocks
        WHERE run_id < $1; 
    "#,
        to_delete,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip(_pool))]
async fn import_users(_pool: &PgPool, run_id: i32) -> Result<(), anyhow::Error> {
    // sqlx::query!(
    //     r#"
    //     INSERT INTO network_monitoring_users (
    //         user_id,
    //         wallet,
    //         replica_set,
    //         run_id,
    //         primarySpID,
    //         secondary1SpID,
    //         secondary2SpID
    //     )
    //     SELECT
    //         user_id,
    //         wallet,
    //         creator_node_endpoint as replica_set,
    //         $1,
    //         primary_id as primarySpID,
    //         secondary_ids[1] as secondary1SpID,
    //         secondary_ids[2] as secondary2SpID
    //     FROM users
    //     WHERE is_current = TRUE;
    // "#,
    //     run_id,
    // )
    // .execute(pool)
    // .await?;

    Ok(())
}

#[tracing::instrument(skip(_pool))]
async fn import_content_nodes(_pool: &PgPool, run_id: i32) -> Result<(), anyhow::Error> {
    Ok(())
}

#[tracing::instrument(skip(_pool))]
async fn import_cids(_pool: &PgPool, run_id: i32) -> Result<(), anyhow::Error> {
    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT metadata_multihash, $1, 'metadata', user_id
    //     FROM users
    //     WHERE metadata_multihash IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT profile_picture, $1, 'image', user_id
    //     FROM users
    //     WHERE profile_picture IS NOT NULL
    //     AND profile_picture != '0'
    //     AND user_id IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT profile_picture_sizes, $1, 'dir', user_id
    //     FROM users
    //     WHERE profile_picture_sizes IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT cover_photo, $1, 'image', user_id
    //     FROM users
    //     WHERE cover_photo IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT cover_photo_sizes, $1, 'dir', user_id
    //     FROM users
    //     WHERE cover_photo_sizes IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT cover_art, $1, 'image', owner_id
    //     FROM tracks
    //     WHERE cover_art IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT cover_art_sizes, $1, 'dir', owner_id
    //     FROM tracks
    //     WHERE cover_art_sizes IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT metadata_multihash, $1, 'metadata', owner_id
    //     FROM tracks
    //     WHERE metadata_multihash IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT download -> 'cid' as cid, $1, 'track', owner_id
    //     FROM tracks
    //     WHERE download -> 'cid' != 'null'
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    // sqlx::query!(r#"
    //     INSERT INTO network_monitoring_cids_from_discovery (cid, run_id, ctype, user_id)
    //     SELECT
    //         jsonb_array_elements(track_segments) -> 'multihash',
    //         $1,
    //         'track',
    //         owner_id
    //     FROM tracks
    //     WHERE track_segments IS NOT NULL
    //     AND is_current = TRUE;
    // "#, run_id).execute(pool).await?;

    Ok(())
}
