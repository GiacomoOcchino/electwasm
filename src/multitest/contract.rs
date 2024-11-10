use cosmwasm_std::{Addr, Coin, Empty, Response, StdResult, Uint128};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    contract::{self, execute, instantiate, query},
    msg::{
        ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg, VoteInfo, VoteListResponse,
        VoteResponse, Voter,
    },
    ContractError,
};

pub struct ElectwasmContract(Addr);

impl ElectwasmContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        commissions: Vec<Coin>,
        voting_fee: u64,
    ) -> StdResult<Self> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                commissions,
                voting_fee,
            },
            &[],
            label,
            None,
        )
        .map(ElectwasmContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn create_proposal(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
        proposal: ExecuteMsg,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &proposal, funds)
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn voters_action(
        &self,
        app: &mut App,
        sender: &Addr,
        action: ExecuteMsg,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &action, &[])
            .map_err(|err| err.downcast().unwrap())
    }
    #[track_caller]
    pub fn vote_proposal(
        &self,
        app: &mut App,
        sender: &Addr,
        action: ExecuteMsg,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &action, &[])
            .map_err(|err| err.downcast().unwrap())
    }
    #[track_caller]
    pub fn close_proposal(
        &self,
        app: &mut App,
        sender: &Addr,
        action: ExecuteMsg,
    ) -> Result<AppResponse, ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &action, &[])
            .map_err(|err| err.downcast().unwrap())
    }

    pub fn query_proposal_info(&self, app: &App, proposal_id: u64) -> StdResult<ProposalResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Proposal { proposal_id })
    }
}
