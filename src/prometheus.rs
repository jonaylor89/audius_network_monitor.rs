use prometheus::IntGaugeVec;

use lazy_static::lazy_static;
use prometheus::register_int_gauge_vec;

lazy_static! {
    pub static ref USER_COUNT_GAUGE: IntGaugeVec = register_int_gauge_vec!(
        "audius_nm_user_count",
        "the number of users on audius",
        &["run_id"]
    )
    .unwrap();
}
