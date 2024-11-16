use std::collections::HashMap;

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

// #[cw_serde]
// pub struct Votes {
//     pub a: u64,
//     pub b: u64,
//     pub c: u64,
//     pub d: u64,
// }

// impl Votes {
//     /// sum of all votes
//     pub fn total(&self) -> u64 {
//         self.a + self.b + self.c + self.d
//     }

//     /// create it with a yes vote for this much
//     pub fn start() -> Self {
//         Votes {
//             a: 0,
//             b: 0,
//             c: 0,
//             d: 0,
//         }
//     }

//     pub fn add_vote(&mut self, vote: Vote, weight: u64) {
//         match vote {
//             Vote::A => self.a += weight,
//             Vote::B => self.b += weight,
//             Vote::C => self.c += weight,
//             Vote::D => self.d += weight,
//         }
//     }
// }


#[cw_serde]
pub struct Votes {
    pub votes: HashMap<usize, u64>, // Mappa l'indice dell'opzione al conteggio dei voti
}

impl Votes {
    /// Inizializza una nuova mappa di voti vuota
    pub fn start(num_options: usize) -> Self {
        let mut votes = HashMap::new();
        for i in 0..num_options {
            votes.insert(i, 0);
        }
        Votes { votes }
    }

    /// Aggiunge un voto all'opzione specificata
    pub fn add_vote(&mut self, option_index: usize, weight: u64) {
        if let Some(count) = self.votes.get_mut(&option_index) {
            *count += weight;
        }
    }

    /// Restituisce il totale di tutti i voti
    pub fn total(&self) -> u64 {
        self.votes.values().sum()
    }
}
// #[cw_serde]
// pub enum Vote {
//     A,
//     B,
//     C,
//     D,
// }
#[cw_serde]
pub enum Vote {
    Option(usize), // Usa l'indice dell'opzione per la votazione
}
// It contains the vote info
#[cw_serde]
pub struct Ballot {
    pub weight: u64,
    pub vote: Vote,
}

pub const STATUS: Item<State> = Item::new("status");

// pub const BALLOTS: Map<(u64, &Addr), Vote> = Map::new("votes");
pub const BALLOTS: Map<(u64, &Addr), usize> = Map::new("votes");


/*Voters info Map<(ID proposal, voter address), can vote? true or false>*/
pub const VOTERS: Map<(u64, &Addr), bool> = Map::new("voters");

#[cw_serde]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub option: Vec<String>,
    pub expires: Expiration,
    pub votes: Votes,
    pub status: ProposalStatus,
    pub proposer: Addr,
    pub winner: Option<String>, // pub fee: Uint128,
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
    /// (designed for handler logic)
    pub fn update_status(&mut self, block: &BlockInfo) {
        self.status = self.current_status(block);
    }
}
/*Proposal info Map<ID proposal, Proposal info>*/
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

#[cw_serde]
pub struct State {
    pub admin: Addr,
    pub commissions: Vec<Coin>, //TODO DA INSERIRE
    pub voting_fee: u64,        //Todo valutare a chi pagare la fee
}

pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
    PROPOSAL_COUNT.save(store, &id)?;
    Ok(id)
}
