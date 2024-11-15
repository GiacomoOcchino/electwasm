use crate::state::{ProposalStatus, Vote, Votes};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    pub commissions: Vec<Coin>, //TODO DA INSERIRE
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
    Close {
        proposal_id: u64,
    },
    Propose {
        title: String,
        description: String,
        option: Vec<String>,
        expires: Expiration,
        // msgs: Vec<CosmosMsg<Empty>>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
    },
}

#[cw_serde]
pub struct VoteInfo {
    pub voter: String,
    pub vote: Vote,
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
    Running { proposal_id: u64 },
    #[returns(ProposalResponse)]
    Proposal { proposal_id: u64 },
    #[returns(ProposalResult)]
    Winner { proposal_id: u64 },
    #[returns(ProposalIdsWithTitlesResponse)]
    AllProposalIds {},
    #[returns(ProposalsByProposerResponse)]
    ProposalByProposer { proposer: Addr },
    // #[returns(VoteListResponse)]
    // GetAllVotes { proposal_id: u64 },
}

#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde]
pub struct ProposalIdsWithTitlesResponse {
    pub proposals: Vec<(u64, String)>,
}

#[cw_serde]
pub struct ProposalsByProposerResponse {
    pub proposals: Vec<(u64, String)>, // ID e titolo delle proposte
}

#[cw_serde]
pub struct ProposalResponse {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub expires: Expiration,
    pub proposer: Addr,
    pub options: Vec<String>,
}

#[cw_serde]
pub struct ProposalResult {
    pub title: String,
    pub description: String,
    pub winner: Option<String>, // Vincitore o None se non c'è ancora un vincitore
}
