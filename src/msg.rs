use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils::{Duration, Expiration};
#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub option: Vec<String>,
    pub expiration: Expiration,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}


#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}
