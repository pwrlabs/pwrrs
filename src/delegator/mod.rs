use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Delegator {
    pub address: String,
    pub shares: u64,
}
