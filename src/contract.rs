#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoteInfo, VoteResponse};
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
        // title: "Che pasta ti piace?".to_string(),
        // description: "Dicci la tua".to_string(),
        // option: vec![
        //     "Norma".to_string(),
        //     "Carbonara".to_string(),
        //     "Gricia".to_string(),
        // ],
        title: msg.title,
        description: msg.description,
        option: msg.option,
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Vote { voter } => to_json_binary(&query_vote(deps, voter)?),
        QueryMsg::Total {} => to_json_binary(&query_proposal_response(deps)?),
    }
}

fn query_vote(deps: Deps, voter: String) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, &voter)?;
    let vote = ballot.map(|b| VoteInfo {
        voter: voter.into(),
        vote: b,
    });
    Ok(VoteResponse { vote })
}

fn query_proposal_response(deps: Deps) -> StdResult<Votes> {
    let state = STATUS.load(deps.storage)?;
    let votes = state.votes;
    Ok(Votes {
        a: votes.a,
        b: votes.b,
        c: votes.c,
        d: votes.d,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{Addr, Timestamp};
    use cosmwasm_std::{DepsMut, MessageInfo};
    use cw_multi_test::{App, ContractWrapper, Executor};
    use cw_utils::Expiration;
    

    use crate::msg::InstantiateMsg;
    // #[track_caller]
    // fn proper_initialization(
    //     deps: DepsMut,
    //     info: MessageInfo,
    //     title: String,
    //     description: String,
    //     option: Vec<String>,
    //     expiration: Expiration,
    // ) {
    //     let mut app = App::default();

    //     let owner = app.api().addr_make("owner").to_string();
    //     let voter1 = app.api().addr_make("voter0001").to_string();
    //     let voter2 = app.api().addr_make("voter0002").to_string();
    //     let voter3 = app.api().addr_make("voter0003").to_string();
    //     let voter4 = app.api().addr_make("voter0004").to_string();
    //     let voter5 = app.api().addr_make("voter0005").to_string();
    //     let voter6 = app.api().addr_make("voter0006").to_string();
    //     let title = "Che pasta ti piace?".to_string();
    //     let description = "Dicci la tua".to_string();
    //     let option = vec![
    //         "Norma".to_string(),
    //         "Carbonara".to_string(),
    //         "Gricia".to_string(),
    //     ];
    //     let instantiate_msg = InstantiateMsg {
    //         title,
    //         description,
    //         option,
    //         expiration,
    //     };
    //    let resp= instantiate(deps, mock_env(), info, instantiate_msg)
    // }

    #[test]
fn test_instantiate_with_valid_message() {
    let mut deps = mock_dependencies();
    let ts = Timestamp::from_nanos(1_000_000_202);
    let msg = InstantiateMsg {
        title: "Che pasta ti piace?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec!["Norma".to_string(), "Carbonara".to_string(), "Gricia".to_string()],
        expiration: Expiration::AtTime(ts.plus_days(2)), // Expires in 2 days
    };
    let app = App::default();

    let owner = app.api().addr_make("owner");

    let info = message_info(&owner,&[]);

    let response = instantiate(deps.as_mut(), mock_env(), info, msg.clone())
        .expect("failed to instantiate");

    // Asserto per verificare la risposta 
    assert_eq!(response, Response::default());

    // Asserto per verificare lo stato salvato
    let state = STATUS.load(&deps.storage).expect("failed to load state");
    assert_eq!(state.title, msg.title);
    assert_eq!(state.description, msg.description);
    assert_eq!(state.option, msg.option);
    assert_eq!(Votes::start(), state.votes); // Verifica che i voti siano inizializzati correttamente
    // assert_eq!(state.admin, info.sender);
    assert_eq!(state.expires, msg.expiration);
    assert_eq!(state.status, Status::Open);
}
    // #[test]
    // fn test_instantiate() {
    //     let mut app = App::default();
    //     let code = ContractWrapper::new(execute, instantiate, query);

    //     let owner = app.api().addr_make("owner").to_string();
    //     let voter1 = app.api().addr_make("voter0001").to_string();
    //     let voter2 = app.api().addr_make("voter0002").to_string();
    //     let voter3 = app.api().addr_make("voter0003").to_string();
    //     let voter4 = app.api().addr_make("voter0004").to_string();
    //     let voter5 = app.api().addr_make("voter0005").to_string();
    //     let voter6 = app.api().addr_make("voter0006").to_string();
    //     let title = "Che pasta ti piace?".to_string();
    //     let description = "Dicci la tua".to_string();
    //     let option = vec![
    //         "Norma".to_string(),
    //         "Carbonara".to_string(),
    //         "Gricia".to_string(),
    //     ];
    //     let instantiate_msg = InstantiateMsg {
    //         title,
    //         description,
    //         option,
    //         expiration,
    //     };
    //     instantiate(deps, mock_env(), info, instantiate_msg)
    // }
}
