use crate::state::ProposalStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    pub commissions: Vec<Coin>,
    pub voting_fee: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Vote {
        vote: usize,
        proposal_id: u64,
    },
    UpdateVoters {
        ask: String,
        add: Vec<String>,
        rmv: Vec<String>,
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
    },
}

#[cw_serde]
pub struct VotesResponse {
    pub counts: Vec<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VotesResponse)]
    Running { proposal_id: u64 },
    #[returns(ProposalResponse)]
    Proposal { proposal_id: u64 },
    #[returns(ProposalResult)]
    Winner { proposal_id: u64 },
    #[returns(VotersResponse)]
    Voters { proposal_id: u64 },
    #[returns(AllProposalsInfoResponse)]
    AllProposals {},
    #[returns(StatusResponse)]
    Status {},
    #[returns(ProposalsByProposerResponse)]
    ProposalsByProposer { proposer: Addr },
}

#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde]
pub struct AllProposalsInfoResponse {
    pub proposals: Vec<(u64, String, ProposalStatus)>,
}

#[cw_serde]
pub struct ProposalsInfoByProposer {
    pub id: u64,
    pub title: String,
    pub status: ProposalStatus,
    pub winner: Option<String>,
}
#[cw_serde]
pub struct ProposalsByProposerResponse {
    pub proposals: Vec<ProposalsInfoByProposer>,
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
    pub winner: Option<String>,
}

#[cw_serde]
pub struct VotersResponse {
    pub allowed_voters: Vec<Addr>,
    pub pending_voters: Vec<Addr>,
    pub has_voted_voters: Vec<Addr>,
}

#[cw_serde]
pub struct StatusResponse {
    pub admin: String,
    pub commissions: Vec<String>,
    pub voting_fee: u64,
}
