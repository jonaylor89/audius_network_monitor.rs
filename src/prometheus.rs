use prometheus::IntGaugeVec;

use lazy_static::lazy_static;
use prometheus::register_int_gauge_vec;

lazy_static! {
    pub(crate) static ref USER_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_user_count",
        "the number of users on audius",
        &["run_id"]
    )
    .unwrap();
    pub(crate) static ref ALL_USER_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_all_user_count",
        "the count of users with this content node in their replica set",
        &["endpoint", "run_id"],
    )
    .unwrap();
    pub(crate) static ref PRIMARY_USER_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_primary_user_count",
        "the count of users with this content node as their primary",
        &["endpoint", "run_id"],
    )
    .unwrap();
    pub(crate) static ref FULLY_SYNCED_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_fully_synced_user_count",
        "the number of users whose content nodes replicas are all in sync",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref PARTIALLY_SYNCED_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_partially_synced_user_count",
        "the number of users whose primary is in sync with only one secondary",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref UNSYNCED_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_unsynced_user_count",
        "the number of users whose primary is out of sync with both secondaries",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref NULL_PRIMARY_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_no_primary_user_count",
        "the number of users whose primary is null",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref UNHEALTHY_REPLICA_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_unhealthy_replica_users_count",
        "the number of users who have an unhealthy replica",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref MISSED_USERS_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_missed_users_count",
        "the number of users that got skipped while indexing content nodes",
        &["endpoint", "run_id"]
    )
    .unwrap();
    pub(crate) static ref INDEXING_DISCOVERY_DURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_indexing_discovery_duration",
        "the amount of time it takes to index the discovery database",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref INDEXING_CONTENT_DURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_indexing_content_duration",
        "the amount of time it takes to index the content node network",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref GENERATING_METRICS_DURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_generating_metrics_duration",
        "the amount of time it takes to generate metrics from the DB",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref TOTAL_JOB_DURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_total_job_duration",
        "the amount of time it takes for an entire network monitoring job to complete",
        &["run_id"],
    )
    .unwrap();
    pub(crate) static ref USER_BATCH_DURATION_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_user_batch_duration",
        "the amount of time it takes to fetch and save a user batch",
        &["run_id", "endpoint"],
    )
    .unwrap();
    pub(crate) static ref USERS_WITH_ALL_FOUNDATION_NODE_REPLICA_SET_GAUGE: IntGaugeVec =
        register_int_gauge_vec!(
            "audius_nm_users_with_all_foundation_node_replica_set",
            "the number of users whose entire replica set is made of foundation nodes",
            &["run_id"]
        )
        .unwrap();
    pub(crate) static ref USERS_WITH_NO_FOUNDATION_NODE_REPLICA_SET_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_users_with_no_foundation_node_replica_set",
        "the number of users whose entire replica set is does not contain any foundation nodes",
        &["run_id"]
    )
    .unwrap();
    pub(crate) static ref FULLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_fully_synced_user_by_primary_count",
        "the number of users whose content nodes replicas are all in sync grouped by primary",
        &["run_id", "endpoint"]
    )
    .unwrap();
    pub(crate) static ref PARTIALLY_SYNCED_USER_BY_PRIMARY_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_partially_synced_user_by_primary_count",
        "the number of users whose primary is in sync with only one secondary grouped by primary",
        &["run_id", "endpoint"]
    )
    .unwrap();
    pub(crate) static ref UNSYNCED_USER_BY_PRIMARY_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_unsynced_user_by_primary_count",
        "the number of users whose primary is out of sync with both secondaries grouped by primary",
        &["run_id", "endpoint"]
    )
    .unwrap();
    pub(crate) static ref FULLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_fully_synced_user_by_replica_count",
        "the number of users whose content nodes replicas are all in sync grouped by replica",
        &["run_id", "endpoint"]
    )
    .unwrap();
    pub(crate) static ref PARTIALLY_SYNCED_USER_BY_REPLICA_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_partially_synced_user_by_replica_count",
        "the number of users whose primary is in sync with only one secondary grouped by replica",
        &["run_id", "endpoint"]
    )
    .unwrap();
    pub(crate) static ref UNSYNCED_USER_BY_REPLICA_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_unsynced_user_by_replica_count",
        "the number of users whose primary is out of sync with both secondaries grouped by replica",
        &["run_id", "endpoint"]
    )
    .unwrap();
}
