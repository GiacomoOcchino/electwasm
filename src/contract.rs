#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATUS};
use crate::{exec, query};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        admin: info.sender,
        commissions: msg.commissions,
        voting_fee: 0,
    };

    STATUS.save(deps.storage, &state)?;

    //TODO Check
    /* Imposta la fee iniziale, ad esempio prelevandola dai fondi del creatore
    let fee = info.funds.get(&denom).cloned().unwrap_or_default();*/
    Ok(Response::default())
    /*Ok(Response::new().add_attribute("message", "contract initialized"))
     */
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
        } => exec::execute_create_proposal(
            deps,
            env,
            info,
            title,
            description,
            option,
            expires,
        ),
        Vote { vote, proposal_id } => exec::execute_vote(deps, env, info, vote, proposal_id),
        UpdateVoters {
            add,
            ask,
            proposal_id,
        } => exec::execute_update_voters(deps, env, info, add, ask, proposal_id),
        Close { proposal_id } => exec::execute_close(deps, env, info, proposal_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        //DONE
        QueryMsg::Vote { voter, proposal_id } => {
            to_json_binary(&query::query_vote(deps, voter, proposal_id)?)
        }
        //DONE
        QueryMsg::Total { proposal_id } => {
            to_json_binary(&query::query_proposal_response(deps, proposal_id)?)
        }

        QueryMsg::Proposal { proposal_id } => {
            to_json_binary(&query::query_proposal(deps, env, proposal_id)?)
        } //TODO
          /*
          QueryMsg::GetAllVotes {proposal_id} => {

              let mut all_votes: Vec<VoteInfo> = vec![];
              BALLOTS
                  .range(deps.storage, None, None, Order::Ascending)
                  .for_each(|item| {
                      let (voter, vote) = item.unwrap();
                      all_votes.push(VoteInfo {
                          voter: voter.to_string(),
                          vote,
                      });
                  });
              to_json_binary(&all_votes)
          }
          */
    }
}
