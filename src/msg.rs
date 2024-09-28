use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_utils:: Expiration;
use crate::state::{Vote, Votes};
#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub option: Vec<String>,
    pub expiration: Expiration,
}

#[cw_serde]
pub enum ExecuteMsg {
    Vote {
        vote: Vote,
    },
    UpdateVoters {
        ask: String,
        add: Vec<String>,
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
    Total { }, 
    #[returns(VoteListResponse)]
    GetAllVotes{}
}


#[cw_serde]
pub struct Voter {
    pub addr: String,
    pub weight: u64,
}
