use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

#[cw_serde]
#[derive(Copy)]
#[repr(u8)]
pub enum ProposalStatus {
    /// proposal was created, voting is started
    Open = 1,
    /// proposal expired
    Closed = 2,
}
#[cw_serde]
pub struct Votes {
    pub counts: Vec<u64>,
}

impl Votes {
    /// Initialize a new empty vote map
    pub fn start(num_options: usize) -> Self {
        Votes {
            counts: vec![0; num_options], // Initialize all counters to zero
        }
    }

    /// Adds a vote to the specified option
    pub fn add_vote(&mut self, option_index: usize, weight: u64) {
        if let Some(count) = self.counts.get_mut(option_index) {
            *count += weight;
        }
    }

    /// Returns the total of all votes
    pub fn total(&self) -> u64 {
        self.counts.iter().sum()
    }
}

#[cw_serde]
pub enum VoterStatus {
    NotAllowed, // User cannot vote
    CanVote,    // User can vote
    HasVoted,   // The user has already voted
}

pub const STATUS: Item<State> = Item::new("status");
pub const VOTERS: Map<(u64, &Addr), VoterStatus> = Map::new("voters");

#[cw_serde]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub option: Vec<String>,
    pub expires: Expiration,
    pub votes: Votes,
    pub status: ProposalStatus,
    pub proposer: Addr,
    pub winner: Option<String>,
}
impl Proposal {
    pub fn current_status(&self, block: &BlockInfo) -> ProposalStatus {
        let mut status = self.status;

        if status == ProposalStatus::Open && self.expires.is_expired(block) {
            status = ProposalStatus::Closed;
        }

        status
    }

    /// update_status sets the status of the proposal to current_status.
    pub fn update_status(&mut self, block: &BlockInfo) {
        self.status = self.current_status(block);
    }
}
/*Proposal info Map<ID proposal, Proposal info>*/
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

#[cw_serde]
pub struct State {
    pub admin: Addr,
    pub commissions: Vec<Coin>, //Fees to be paid to the contract instantiator to create voting proposals
    pub voting_fee: u64,        //Fees to be paid to the contract instantiator for voting
}

pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
    PROPOSAL_COUNT.save(store, &id)?;
    Ok(id)
}
