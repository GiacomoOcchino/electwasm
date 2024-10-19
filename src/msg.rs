use std::collections::BTreeMap;

use crate::state::{Proposal, Vote, Votes};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty};
use cw_utils::Expiration;
// #[cw_serde]
// pub struct InstantiateMsg {
//     pub count: String,
//     pub description: String,
//     pub option: Vec<String>,
//     pub expiration: Expiration,
// }
#[cw_serde]
pub struct InstantiateMsg {
    pub count: u64,
    pub proposals: BTreeMap<u64, Proposal>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Vote {
    //     vote: Vote,
    // },
    UpdateVoters {
        ask: String,
        add: Vec<String>,
        proposal_id: u64,
    },
    Propose {
        title: String,
        description: String,
        option: Vec<String>,
        expires: Expiration,
        msgs: Vec<CosmosMsg<Empty>>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
    },
}

#[cw_serde]
pub struct VoteInfo {
    pub voter: String,
    pub vote: Vote,
    // pub weight: u64,
}
#[cw_serde]
pub struct VoteListResponse {
    pub votes: Vec<VoteInfo>,
}
#[cw_serde]
pub struct VoteResponse {
    pub vote: VoteInfo,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VoteResponse)]
    Vote { voter: String },
    #[returns(Votes)]
    Total {},
    #[returns(VoteListResponse)]
    GetAllVotes {},
}

#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}
