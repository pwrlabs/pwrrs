use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Delegator {
    pub address: String,
    pub validator_address: String,
    pub shares: u64,
    pub delegated_pwr: u64,
}
