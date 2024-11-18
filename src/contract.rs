#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATUS};
use crate::{exec, query};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        admin: info.sender,
        commissions: msg.commissions,
        voting_fee: 0,
    };

    STATUS.save(deps.storage, &state)?;

    Ok(Response::default())
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::{Close, Propose, UpdateVoters, Vote};
    match msg {
        Propose {
            title,
            description,
            option,
            expires,
        } => exec::execute_create_proposal(deps, env, info, title, description, option, expires),
        Vote { vote, proposal_id } => exec::execute_vote(deps, env, info, vote, proposal_id),
        UpdateVoters {
            add,
            ask,
            rmv,
            proposal_id,
        } => exec::execute_update_voters(deps, env, info, add, ask,rmv, proposal_id),
        Close { proposal_id } => exec::execute_close(deps, env, info, proposal_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        /*QueryMsg::Vote { voter, proposal_id } => {
            to_json_binary(&query::query_vote(deps, voter, proposal_id)?)
        }*/
        QueryMsg::Running { proposal_id } => {
            to_json_binary(&query::query_proposal_running_response(deps, proposal_id)?)
        }
        QueryMsg::Winner { proposal_id } => {
            to_json_binary(&query::query_proposal_result(deps, proposal_id)?)
        }

        QueryMsg::Proposal { proposal_id } => {
            to_json_binary(&query::query_proposal(deps, env, proposal_id)?)
        }
        QueryMsg::ProposalByProposer { proposer } => {
            to_json_binary(&query::query_proposals_by_proposer(deps, proposer)?)
        }
        QueryMsg::AllProposalIds {} => {
            to_json_binary(&query::query_all_proposal_ids_with_titles(deps)?)
        }
        QueryMsg::Voters { proposal_id } => {
            to_json_binary(&query::query_voters(deps, env, proposal_id)?)
        }
    }
}
