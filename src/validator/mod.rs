use std::net::IpAddr;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub ip: IpAddr,
    #[serde(default)]
    pub bad_actor: bool,
    pub voting_power: u128,
    pub total_shares: u128,
    pub delegators_count: u32,
    pub status: String,
}
