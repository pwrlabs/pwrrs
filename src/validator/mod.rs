use serde::{Deserialize};
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub ip: IpAddr,
    pub bad_actor: bool,
    pub voting_power: u64,
    pub total_shares: u64,
    pub delegators_count: u32,
    pub is_active: bool,
}