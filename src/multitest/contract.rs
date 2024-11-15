use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    contract::{ execute, instantiate, query},
    msg::{
        ExecuteMsg, InstantiateMsg, ProposalIdsWithTitlesResponse, ProposalResponse, ProposalResult,ProposalsByProposerResponse, QueryMsg },
    state::Votes,
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
    pub fn query_proposal_running_response(&self, app: &App, proposal_id: u64) -> StdResult<Votes> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Running { proposal_id })
    }
    pub fn query_proposal_winner(&self, app: &App, proposal_id: u64) -> StdResult<ProposalResult> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Winner { proposal_id })
    }
    pub fn query_all_proposal(&self, app: &App) -> StdResult<ProposalIdsWithTitlesResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::AllProposalIds { })
    }
    pub fn query_proposal_by_proposer(&self, app: &App, proposer: Addr) -> StdResult<ProposalsByProposerResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::ProposalByProposer { proposer })
    }
}
