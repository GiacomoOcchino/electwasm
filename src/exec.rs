use cosmwasm_std::{
    attr, coin, from_json, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Empty,
    Env, MessageInfo, Order, Response, StdError, StdResult, Timestamp,
};
use cw_utils::Expiration;

use crate::{
    state::{
        next_id, Proposal, ProposalStatus, Vote, Votes, ADMINS, BALLOTS, PROPOSALS, STATUS, VOTERS,
    },
    ContractError,
};
pub fn execute_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    option: Vec<String>,
    expires: Expiration,
    msgs: Vec<CosmosMsg>,
    // we ignore earliest
) -> Result<Response<Empty>, ContractError> {
    /*let status = STATUS.load(deps.storage)?;*/
    // create a proposal
    let mut prop = Proposal {
        title,
        description,
        expires,
        option,
        msgs,
        status: ProposalStatus::Open,
        votes: Votes::start(),
        proposer: info.sender.clone(),
        // fee: None,
    };
    prop.update_status(&env.block); //TODO Check
    let id = next_id(deps.storage)?;
    PROPOSALS.save(deps.storage, id, &prop)?;

    /*
    // add the first yes vote from voter
    let ballot = Ballot {
        weight: vote_power,
        vote: Vote::Yes,
    };


    BALLOTS.save(deps.storage, (id, &info.sender), &ballot)?;
    */
    // add the first voter to the proposal?
    Ok(Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status)))
}

pub fn execute_update_voters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: Vec<String>,
    ask: String,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let attributes = vec![
        attr("action", "update_voters"),
        attr("proposal", proposal_id.to_string()),
        attr("added", add.len().to_string()),
        attr("ask to join", ask.clone()),
        attr("sender", &info.sender),
    ];

    //Check if propose exist
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    //Check if is OPEN
    if ![ProposalStatus::Open].contains(&prop.status) {
        return Err(ContractError::NotOpen {});
    }

    // make the local update
    update_voters(deps, info.sender, proposal_id, add, ask)?;
    // call all registered hooks

    Ok(Response::new().add_attributes(attributes))
}

// the logic from execute_update_voters extracted for easier import
pub fn update_voters(
    deps: DepsMut,
    sender: Addr,
    proposal_id: u64,
    to_add: Vec<String>,
    to_ask: String,
) -> Result<Response, ContractError> {
    // validate_unique_voters(&mut to_add)?;
    let to_add = to_add; // let go of mutability

    // ADMIN.assert_admin(deps.as_ref(), &sender)?;
    if !to_ask.is_empty() {
        let insert_addr = deps.api.addr_validate(&to_ask)?;

        VOTERS.save(deps.storage, (proposal_id, &insert_addr), &false)?;
    }
    if !to_add.is_empty() {
        //Reference to proposal
        let prop = PROPOSALS.load(deps.storage, proposal_id)?;
        if prop.proposer == sender {
            for voter in to_add {
                let update_addr = deps.api.addr_validate(&voter)?;
                VOTERS.update(
                    deps.storage,
                    (proposal_id, &update_addr),
                    |old| -> StdResult<_> {
                        Ok(match old {
                            Some(true) => true,  // Se è già true, lo lasciamo così
                            Some(false) => true, // Se è false, lo cambiamo in true
                            None => true,        // Se non esiste, lo inseriamo con valore true
                        })
                    },
                )?;
            }
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    Ok(Response::new())
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vote: Vote,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    //Check if propose exist
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    //Check if is OPEN
    if ![ProposalStatus::Open].contains(&prop.status) {
        return Err(ContractError::Expired {});
    }
    let voter = VOTERS.may_load(deps.storage, (proposal_id, &info.sender))?;
    match voter {
        Some(true) => {
            // L'utente può votare
            BALLOTS.update(deps.storage, (proposal_id, &info.sender), |bal| match bal {
                Some(_) => Err(ContractError::AlreadyVoted {}),
                None => Ok(vote.clone()),
            })?;
            prop.votes.add_vote(vote, 1);
            prop.update_status(&env.block);
            PROPOSALS.save(deps.storage, proposal_id, &prop)?;

            Ok(Response::new()
                .add_attribute("action", "vote")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status)))
        }
        Some(false) => {
            // L'utente non può votare
            return Err(ContractError::Unauthorized {});
        }
        None => {
            // L'utente non è stato trovato
            Err(ContractError::Unauthorized {})
        }
    }
    // Check if voter already vote
}
