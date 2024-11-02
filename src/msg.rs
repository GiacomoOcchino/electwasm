use std::collections::BTreeMap;

use crate::state::{Proposal, ProposalStatus, Vote, Votes};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    pub accepted_tokens: Vec<String>,
    pub proposal_commission: u128,
    pub voting_fee: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Vote {
        vote: Vote,
        proposal_id: u64,
    },
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
    Vote { voter: String, proposal_id: u64 },
    #[returns(Votes)]
    Total { proposal_id: u64 },
    #[returns(ProposalResponse)]
    Proposal { proposal_id: u64 },
    // #[returns(VoteListResponse)]
    // GetAllVotes { proposal_id: u64 },
}

#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde]
pub struct ProposalResponse<T = Empty> {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub msgs: Vec<CosmosMsg<T>>,
    pub status: ProposalStatus,
    pub expires: Expiration,
    pub proposer: Addr,
    // pub deposit: Option<DepositInfo>,
}
