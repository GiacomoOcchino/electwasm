use cosmwasm_std::{
    attr, Addr, BankMsg, Coin, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Storage,
    Uint128,
};
use cw_utils::Expiration;

use crate::{
    state::{next_id, Proposal, ProposalStatus, VoterStatus, Votes, PROPOSALS, STATUS, VOTERS},
    ContractError,
};
pub fn execute_create_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    option: Vec<String>,
    expires: Expiration,
) -> Result<Response<Empty>, ContractError> {
    let state = STATUS.load(deps.storage)?;
    let owner = state.admin;
    let commissions = state.commissions;

    if option.is_empty() {
        return Err(ContractError::NoOptionsProvided {});
    }
    /* if needed is possible to choose max options number
    const MAX_OPTIONS: usize = 10;
    if option.len() > MAX_OPTIONS {
        return Err(ContractError::TooManyOptions {});
    }*/

    // Check if the proposer sent any funds
    if info.funds.is_empty() {
        return Err(ContractError::MissingPayment {});
    }
    // Find the matching commission for the provided funds
    let mut matching_commission: Option<&Coin> = None;
    let mut coin_sended = Uint128::new(0);

    for coin in info.funds.iter() {
        for allowed_commission in commissions.iter() {
            if coin.denom == allowed_commission.denom {
                matching_commission = Some(allowed_commission);
                coin_sended = coin.amount;
                break; // Early exit if a matching commission is found
            }
        }
    }
    // Handle cases where no matching commission is found
    if matching_commission.is_none() {
        return Err(ContractError::UnsupportedToken {});
    }
    let commission = matching_commission.unwrap();
    let funds = coin_sended;

    // Check if the sent funds are enough for the commission
    if funds < commission.amount {
        return Err(ContractError::InsufficientFunds {
            funds,
            commission: commission.amount,
        });
    }

    // Build the commission message
    let commission_msg = BankMsg::Send {
        to_address: owner.into_string(),
        amount: vec![commission.clone()], // Use the matching coin from `info.funds`
    };

    let mut resp = Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("status", format!("{:?}", ProposalStatus::Open));

    // Add the commission message only if there's a commission
    if !commission.amount.is_zero() {
        resp = resp
            .add_message(commission_msg)
            .add_attribute("commission_payer", info.sender.clone());
    }
    let num_options = option.len();
    let votes = Votes::start(num_options);
    // create a proposal
    let mut prop = Proposal {
        title,
        description,
        expires,
        option,
        status: ProposalStatus::Open,
        votes,
        proposer: info.sender.clone(),
        winner: None,
    };
    prop.update_status(&env.block);
    let id = next_id(deps.storage)?;
    PROPOSALS.save(deps.storage, id, &prop)?;
    resp = resp
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status));

    Ok(resp)
}

pub fn execute_update_voters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    add: Vec<String>,
    ask: String,
    rmv: Vec<String>,
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
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;

    //Check if is OPEN
    if ![ProposalStatus::Open].contains(&prop.status) {
        return Err(ContractError::Expired {});
    }

    // make the local update
    update_voters(deps, info.sender, proposal_id, add, ask, rmv)?;

    Ok(Response::new().add_attributes(attributes))
}

// the logic from execute_update_voters extracted for easier import

pub fn update_voters(
    deps: DepsMut,
    sender: Addr,
    proposal_id: u64,
    to_add: Vec<String>,
    to_ask: String,
    to_remove: Vec<String>, // New parameter to remove voters
) -> Result<Response, ContractError> {
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    // Removing voters only if they have not already voted
    if !to_remove.is_empty() {
        if prop.proposer == sender {
            for voter in to_remove {
                let remove_addr = deps.api.addr_validate(&voter)?;
                let status = VOTERS.may_load(deps.storage, (proposal_id, &remove_addr))?;
                match status {
                    Some(VoterStatus::CanVote) | Some(VoterStatus::NotAllowed) => {
                        // Remove only if voter did not vote
                        VOTERS.remove(deps.storage, (proposal_id, &remove_addr));
                    }
                    Some(VoterStatus::HasVoted) => {
                        // You cannot remove a voter who has already voted.
                        return Err(ContractError::CannotRemoveVoter {});
                    }
                    None => {
                        // Do nothing if the voter is not present
                    }
                }
            }
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    // Add a voter to the request list only if they are not already there
    if !to_ask.is_empty() {
        let insert_addr = deps.api.addr_validate(&to_ask)?;
        let status = VOTERS.may_load(deps.storage, (proposal_id, &insert_addr))?;
        if status.is_none() {
            // Add to request list only if address is not present
            VOTERS.save(
                deps.storage,
                (proposal_id, &insert_addr),
                &VoterStatus::NotAllowed,
            )?;
        }
    }
    // Rest of the logic to add voters
    if !to_add.is_empty() {
        if prop.proposer == sender {
            for voter in to_add {
                let update_addr = deps.api.addr_validate(&voter)?;
                VOTERS.update(
                    deps.storage,
                    (proposal_id, &update_addr),
                    |old| -> StdResult<_> {
                        Ok(match old {
                            Some(VoterStatus::NotAllowed) => VoterStatus::CanVote,
                            Some(VoterStatus::CanVote) | Some(VoterStatus::HasVoted) => {
                                old.unwrap()
                            }
                            None => VoterStatus::CanVote,
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
    vote: usize,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    //Check if propose exist
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    prop.update_status(&env.block);

    //Check if is OPEN
    if ![ProposalStatus::Open].contains(&prop.status) {
        return Err(ContractError::Expired {});
    }

    let voter_status = VOTERS.may_load(deps.storage, (proposal_id, &info.sender))?;
    match voter_status {
        Some(VoterStatus::CanVote) => {
            // Update status to "HasVoted"
            VOTERS.save(
                deps.storage,
                (proposal_id, &info.sender),
                &VoterStatus::HasVoted,
            )?;
            // Verify that the option index is valid
            if vote >= prop.option.len() {
                return Err(ContractError::InvalidOption {});
            }
            prop.votes.add_vote(vote, 1);
            PROPOSALS.save(deps.storage, proposal_id, &prop)?;

            Ok(Response::new()
                .add_attribute("action", "vote")
                .add_attribute("sender", info.sender)
                .add_attribute("proposal_id", proposal_id.to_string())
                .add_attribute("status", format!("{:?}", prop.status)))
        }
        Some(VoterStatus::HasVoted) => {
            // The user has already voted
            return Err(ContractError::AlreadyVoted {});
        }
        _ => {
            // User cannot vote or is not on the list
            Err(ContractError::Unauthorized {})
        }
    }
}

pub fn execute_close(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    // Load proposal from storage
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    prop.update_status(&env.block);
    // Only the proposer can close the proposal
    if prop.proposer != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    // Check if the proposal has expired
    if prop.expires.is_expired(&env.block) {
        // Count the number of participants for the calculation of the quorum
        let participant_count = count_participants(deps.storage, proposal_id);
        // Define the quorum percentage (example: 60%)
        let quorum_percentage: f64 = 0.6;
        let quorum = (participant_count as f64 * quorum_percentage).ceil() as u64;
        // Calculate the total number of votes cast
        let total_votes = prop.votes.total();

        if total_votes >= quorum {
            // Number of votes for the option with the most votes
            let max_votes = prop.votes.counts.iter().cloned().max().unwrap_or(0);
            // Filter options that have the maximum number of votes
            let winners: Vec<usize> = prop
                .votes
                .counts
                .iter()
                .enumerate()
                .filter(|&(_, &count)| count == max_votes && max_votes > 0)
                .map(|(index, _)| index)
                .collect();

            if winners.len() > 1 {
                // Map the indexes of the tied options and concatenate the options
                let tied_options: Vec<String> = winners
                    .iter()
                    .filter_map(|&index| prop.option.get(index))
                    .cloned()
                    .collect();

                let concatenated_tied_options = tied_options.join(", ");
                // Update the proposal status with the tie
                prop.status = ProposalStatus::Closed;
                prop.winner = Some(format!("Tie: {}", concatenated_tied_options));
                PROPOSALS.save(deps.storage, proposal_id, &prop)?;

                return Ok(Response::new()
                    .add_attribute("action", "close_proposal")
                    .add_attribute("status", "closed")
                    .add_attribute("winner", concatenated_tied_options));
            } else if let Some(&winner_index) = winners.first() {
                // Check if the index is valid within the options array
                if let Some(winner_text) = prop.option.get(winner_index) {
                    // Update the proposal with the winner
                    prop.status = ProposalStatus::Closed;
                    prop.winner = Some(winner_text.clone());
                    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

                    return Ok(Response::new()
                        .add_attribute("action", "close_proposal")
                        .add_attribute("status", "closed")
                        .add_attribute("winner", winner_text));
                } else {
                    return Err(ContractError::InvalidWinner {});
                }
            } else {
                prop.status = ProposalStatus::Closed;
                prop.winner = Some("No valid votes".to_string());
                PROPOSALS.save(deps.storage, proposal_id, &prop)?;
                return Ok(Response::new()
                        .add_attribute("action", "close_proposal")
                        .add_attribute("status", "closed")
                        .add_attribute("winner", "No valid votes"));
            }
        } else {
            prop.status = ProposalStatus::Closed;
            prop.winner = Some("Quorum not reached".to_string());
            PROPOSALS.save(deps.storage, proposal_id, &prop)?;
            return Ok(Response::new()
                        .add_attribute("action", "close_proposal")
                        .add_attribute("status", "closed")
                        .add_attribute("winner", "Quorum not reached"));
        }
    } else {
        return Err(ContractError::NotExpired {});
    }
}

pub fn count_participants(storage: &dyn Storage, proposal_id: u64) -> u64 {
    VOTERS
        .prefix(proposal_id)
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .filter(|result| match result {
            Ok((_, status)) => matches!(status, VoterStatus::CanVote), // Considera solo quelli con `true`
            Err(_) => false,
        })
        .count() as u64
}
