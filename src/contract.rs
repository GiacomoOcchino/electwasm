#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Ballot, State, Status, Vote, Votes, BALLOTS, STATUS};
/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:voting-system";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        title: "Che pasta ti piace?".to_string(),
        description: "Dicci la tua".to_string(),
        option: vec![
            "Norma".to_string(),
            "Carbonara".to_string(),
            "Gricia".to_string(),
        ],
        votes: Votes::start(),
        // risultati: vec![],
        admin: info.sender.clone(), // L'amministratore è chi crea il contratto
        expires: msg.expiration,
        status: Status::Open,
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
    match msg {
        ExecuteMsg::Vote { vote } => execute_vote(deps, env, info, vote),
    }
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vote: Vote,
) -> Result<Response<Empty>, ContractError> {
    let mut state = STATUS.load(deps.storage)?;
    // Check if proposal is closed
    if ![Status::Open].contains(&state.status) {
        return Err(ContractError::Expired {});
    }
    // Check if voter already vote

    BALLOTS.update(deps.storage, &info.sender, |bal| match bal {
        Some(_) => Err(ContractError::AlreadyVoted {}),
        None => Ok(vote.clone()),
    })?;
    state.votes.add_vote(vote, 1);
    state.update_status(&env.block);
    STATUS.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("sender", info.sender)
        .add_attribute("status", format!("{:?}", state.status)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
