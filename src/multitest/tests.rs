use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::App;

use crate::{
    msg::{ProposalResponse, QueryMsg},
    state::{Proposal, Vote, Votes, STATUS},
    ContractError,
};

use super::contract::ElectwasmContract;

const UJUNO: &str = "ujunox";
const UATOM: &str = "uatom";
