use ahash::AHashMap;
use cogs_gamedev::grids::ICoord;
use serde::{Deserialize, Serialize};

use crate::simulator::transport::{Cable, OmniversalConnector};

#[derive(Clone, Serialize, Deserialize)]
pub struct Solution {
    /// Level key this solution is for
    pub level_key: String,
    /// Cable layout
    pub cables: AHashMap<ICoord, Cable>,

    pub left: OmniversalConnector,
    pub right: OmniversalConnector,

    /// If this is Some, the level is solved!
    pub metrics: Option<Metrics>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_cycles: u64,
    pub min_cycles: u64,
    pub crossovers: u64,
}
