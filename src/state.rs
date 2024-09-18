use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty, StdResult, Storage, Timestamp,BlockInfo};

use cw_storage_plus::{Item, Map};
use cw_utils::{Duration, Expiration};

#[cw_serde]
#[derive(Copy)]
#[repr(u8)]
pub enum Status {
    /// proposal was created, voting is started
    Open = 1,
    /// proposal expired
    Closed = 2,
}

#[cw_serde]
pub struct Votes {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

impl Votes {
    /// sum of all votes
    pub fn total(&self) -> u64 {
        self.a + self.b + self.c + self.d
    }

    /// create it with a yes vote for this much
    pub fn start() -> Self {
        Votes {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    pub fn add_vote(&mut self, vote: Vote, weight: u64) {
        match vote {
            Vote::A => self.a += weight,
            Vote::B => self.b += weight,
            Vote::C => self.c += weight,
            Vote::D => self.d += weight,
        }
    }
}

#[cw_serde]
pub enum Vote {
    /// Marks support for the proposal.
    A,
    /// Marks opposition to the proposal.
    B,
    /// Marks participation but does not count towards the ratio of support / opposed
    C,
    /// Veto is generally to be treated as a No vote. Some implementations may allow certain
    /// voters to be able to Veto, or them to be counted stronger than No in some way.
    D,
}
// It contains the vote info
#[cw_serde]
pub struct Ballot {
    pub weight: u64,
    pub vote: Vote,
}
#[cw_serde]
pub struct Config {
    // pub threshold: Threshold,
    pub total_weight: u64,
    pub max_voting_period: Duration,
}
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATUS: Item<State> = Item::new("status");

// MSG

// pub const BALLOTS: Map< &Addr, Ballot> = Map::new("votes");
pub const BALLOTS: Map<&Addr, Vote> = Map::new("votes");
pub const ADMINS: Map<&Addr, Timestamp> = Map::new("admins");
pub const VOTERS: Map<&Addr, u64> = Map::new("voters");
// pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
// multiple-item maps
// pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");

// #[cw_serde]
// pub struct State {
//     // pub proposal: Vec<Proposal>,
//     pub votes: Vec<Vote>,
//     // pub results: Vec<Risultato>,
//     pub admin: Addr,
//     pub expires: Expiration,

// }

#[cw_serde]
pub struct State {
    pub title: String,
    pub description: String,
    pub option: Vec<String>,
    pub expires: Expiration,
    pub votes: Votes,
    // pub voter: Vec<Addr>,
    pub status: Status,
    pub admin: Addr,
    // pub start_height: u64,
    // pub msgs: Vec<CosmosMsg<Empty>>,
    // pub deposit: Option<DepositInfo>,
}
impl State {
    pub fn current_status(&self, block: &BlockInfo) -> Status {
        let mut status = self.status;

        
        if status == Status::Open  || self.expires.is_expired(block) {
            status = Status::Closed;
        }

        status
    }

    /// update_status sets the status of the proposal to current_status.
    /// (designed for handler logic)
    pub fn update_status(&mut self, block: &BlockInfo) {
        self.status = self.current_status(block);
    }
}
