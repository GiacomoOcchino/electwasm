use cosmwasm_std::{
    attr, Addr, BankMsg, Coin, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Storage,
    Uint128,
};
use cw_utils::Expiration;

use crate::{
    state::{next_id, Proposal, ProposalStatus, Vote, Votes, BALLOTS, PROPOSALS, STATUS, VOTERS},
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
                // found_to_send = coin;
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

    // create a proposal
    let mut prop = Proposal {
        title,
        description,
        expires,
        option,
        status: ProposalStatus::Open,
        votes: Votes::start(),
        proposer: info.sender.clone(),
        winner: None,
    };
    prop.update_status(&env.block); //TODO Check
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
    // call all registered hooks

    Ok(Response::new().add_attributes(attributes))
}

// the logic from execute_update_voters extracted for easier import
// pub fn update_voters(
//     deps: DepsMut,
//     sender: Addr,
//     proposal_id: u64,
//     to_add: Vec<String>,
//     to_ask: String,
// ) -> Result<Response, ContractError> {
//     // validate_unique_voters(&mut to_add)?;
//     let to_add = to_add; // let go of mutability

//     // ADMIN.assert_admin(deps.as_ref(), &sender)?;
//     if !to_ask.is_empty() {
//         let insert_addr = deps.api.addr_validate(&to_ask)?;

//         VOTERS.save(deps.storage, (proposal_id, &insert_addr), &false)?;
//     }
//     if !to_add.is_empty() {
//         //Reference to proposal
//         let prop = PROPOSALS.load(deps.storage, proposal_id)?;
//         if prop.proposer == sender {
//             for voter in to_add {
//                 let update_addr = deps.api.addr_validate(&voter)?;
//                 VOTERS.update(
//                     deps.storage,
//                     (proposal_id, &update_addr),
//                     |old| -> StdResult<_> {
//                         Ok(match old {
//                             Some(true) => true,  // Se è già true, lo lasciamo così
//                             Some(false) => true, // Se è false, lo cambiamo in true
//                             None => true,        // Se non esiste, lo inseriamo con valore true
//                         })
//                     },
//                 )?;
//             }
//         } else {
//             return Err(ContractError::Unauthorized {});
//         }
//     }

//     Ok(Response::new())
// }

pub fn update_voters(
    deps: DepsMut,
    sender: Addr,
    proposal_id: u64,
    to_add: Vec<String>,
    to_ask: String,
    to_remove: Vec<String>, // Nuovo parametro per rimuovere i votanti
) -> Result<Response, ContractError> {
    if !to_remove.is_empty() {
        let prop = PROPOSALS.load(deps.storage, proposal_id)?;
        if prop.proposer == sender {
            for voter in to_remove {
                let remove_addr = deps.api.addr_validate(&voter)?;
                VOTERS.remove(deps.storage, (proposal_id, &remove_addr)); // Rimozione completa
            }
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    if !to_ask.is_empty() {
        let insert_addr = deps.api.addr_validate(&to_ask)?;

        VOTERS.save(deps.storage, (proposal_id, &insert_addr), &false)?;
    }
    // Resto della logica per aggiungere votanti
    if !to_add.is_empty() {
        let prop = PROPOSALS.load(deps.storage, proposal_id)?;
        if prop.proposer == sender {
            for voter in to_add {
                let update_addr = deps.api.addr_validate(&voter)?;
                VOTERS.update(
                    deps.storage,
                    (proposal_id, &update_addr),
                    |old| -> StdResult<_> {
                        Ok(match old {
                            Some(true) => true,
                            Some(false) => true,
                            None => true,
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
    prop.update_status(&env.block);
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
            // prop.update_status(&env.block);
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

pub fn execute_close(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    // Carica la proposta dallo storage
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    prop.update_status(&env.block);

    // Solo il proposer pu
    if prop.proposer != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Controlla se la proposta è scaduta
    if prop.expires.is_expired(&env.block) {
        // Conta il numero di partecipanti per il calcolo del quorum
        let participant_count = count_participants(deps.storage, proposal_id);

        // Definisci la percentuale del quorum (esempio: 60%)
        let quorum_percentage: f64 = 0.6;
        let quorum = (participant_count as f64 * quorum_percentage).ceil() as u64;

        // Calcola il numero totale dei voti espressi
        let total_votes = prop.votes.total();

        if total_votes >= quorum {
            // Mappatura degli indici dei voti con i conteggi corrispondenti
            let vote_counts = [
                (0, prop.votes.a),
                (1, prop.votes.b),
                (2, prop.votes.c),
                (3, prop.votes.d),
            ];

            // Trova l'indice dell'opzione con il massimo numero di voti
            let winner_index = vote_counts
                .iter()
                .max_by_key(|&(_, count)| count)
                .map(|&(index, count)| if count > 0 { Some(index) } else { None })
                .flatten();

            match winner_index {
                Some(index) => {
                    // Verifica se l'indice è valido all'interno dell'array delle opzioni
                    if let Some(winner_text) = prop.option.get(index) {
                        // Aggiorna la proposta con il vincitore e salva
                        prop.status = ProposalStatus::Closed;
                        prop.winner = Some(winner_text.clone());
                        PROPOSALS.save(deps.storage, proposal_id, &prop)?;

                        // Costruisci la risposta con il testo dell'opzione vincente
                        let response = Response::new()
                            .add_attribute("action", "close_proposal")
                            .add_attribute("status", "closed")
                            .add_attribute("winner", winner_text);

                        Ok(response)
                    } else {
                        Err(ContractError::InvalidWinner {})
                    }
                }
                None => Err(ContractError::NoVote {}),
            }
        } else {
            Err(ContractError::QuorumNotReached {})
        }
    } else {
        Err(ContractError::NotExpired {})
    }
}

pub fn count_participants(storage: &dyn Storage, proposal_id: u64) -> u64 {
    let prefix = proposal_id;
    VOTERS
        .prefix(prefix)
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .filter(|result| match result {
            Ok((_, can_vote)) => *can_vote, // Considera solo quelli con `true`
            Err(_) => false,
        })
        .count() as u64
}
