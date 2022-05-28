use std::time::Instant;

use chariot_core::player::choices::Stat;

#[derive(Clone)]
pub struct StatsChange {
    pub stat: Stat,
    pub multiplier: f64,
    pub expiration_time: Instant,
}
