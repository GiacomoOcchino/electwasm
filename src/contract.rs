#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{
    attr, coin, to_json_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Response, StdError, StdResult, Timestamp,
};
use cw_multi_test::App;
use cw_utils::Expiration;
use std::collections::BTreeMap;
use tests::setup_test_case;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoteInfo, VoteResponse};
use crate::state::{
    next_id, Proposal, ProposalStatus, State, Vote, Votes, BALLOTS, PROPOSALS, STATUS, VOTERS,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        proposals: msg.proposals,
    };
    STATUS.save(deps.storage, &state)?;

    //TODO Check
    /* Imposta la fee iniziale, ad esempio prelevandola dai fondi del creatore
    let fee = info.funds.get(&denom).cloned().unwrap_or_default();*/
    Ok(Response::default())
    /*Ok(Response::new().add_attribute("message", "contract initialized"))
     */
}
//OLD
/*#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        title: msg.title,
        description: msg.description,
        option: msg.option,
        votes: Votes::start(),
        admin: info.sender.clone(), // L'amministratore è chi crea il contratto
        expires: msg.expiration,
        status: Status::Open,
    };
    STATUS.save(deps.storage, &state)?;
    VOTERS.save(deps.storage, &info.sender.clone(), &true)?;
    Ok(Response::default())
}
*/
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose {
            title,
            description,
            option,
            expires,
            msgs,
        } => execute_propose(deps, env, info, title, description, option, expires, msgs),
        ExecuteMsg::Vote { vote, proposal_id } => execute_vote(deps, env, info, vote, proposal_id),
        ExecuteMsg::UpdateVoters {
            add,
            ask,
            proposal_id,
        } => execute_update_voters(deps, env, info, add, ask, proposal_id),
    }
}

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        //DONE
        QueryMsg::Vote { voter, proposal_id } => {
            to_json_binary(&query_vote(deps, voter, proposal_id)?)
        }
        //DONE
        QueryMsg::Total { proposal_id } => {
            to_json_binary(&query_proposal_response(deps, proposal_id)?)
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
/*
fn query_vote(deps: Deps, voter: String) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, &voter)?;

    let vote = ballot.map(|b| VoteInfo {
        voter: voter.into(),
        vote: b,
    });
    Ok(VoteResponse { vote })
}
*/
fn query_vote(deps: Deps, voter: String, proposal_id: u64) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, (proposal_id, &voter))?;

    let vote = match ballot {
        Some(b) => VoteInfo {
            voter: voter.into(),
            vote: b,
        },
        None => {
            // Gestisci il caso in cui il voto non è stato trovato
            return Err(StdError::generic_err("Vote not found"));
        }
    };

    Ok(VoteResponse { vote })
}

fn query_proposal_response(deps: Deps, proposal_id: u64) -> StdResult<Votes> {
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    let votes = prop.votes;
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
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{from_json, Addr, Timestamp};
    use cw_multi_test::App;
    use cw_utils::Expiration;

    use crate::msg::InstantiateMsg;

    // this will set up the instantiation for other tests
    #[track_caller]
    pub fn setup_test_case(
        deps: DepsMut,
        info: MessageInfo,
        count: u64,
        proposals: BTreeMap<u64, Proposal>,
    ) -> Result<Response<Empty>, ContractError> {
        // Instantiate a contract with voters
        let instantiate_msg = InstantiateMsg { count, proposals };
        instantiate(deps, mock_env(), info, instantiate_msg)
    }

    #[test]
    fn test_instantiate_with_valid_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        // let ts = Timestamp::from_nanos(env.block.time.nanos()); // Mock timestamp
        let msg = InstantiateMsg {
            count: 0,
            proposals: BTreeMap::new(),
        };
        let app = App::default();

        let owner = app.api().addr_make("owner");

        let info = message_info(&owner, &[]);

        let response = instantiate(deps.as_mut(), mock_env(), info, msg.clone())
            .expect("failed to instantiate");

        // Asserto per verificare la risposta
        assert_eq!(response, Response::default());

        // Asserto per verificare lo stato salvato
        let state = STATUS.load(&deps.storage).expect("failed to load state");
        assert_eq!(state.count, msg.count);
        assert_eq!(state.proposals, msg.proposals);
        // assert_eq!(state.option, msg.option);
        // assert_eq!(Votes::start(), state.votes); // Verifica che i voti siano inizializzati correttamente
        // assert_eq!(state.expires, msg.expiration);
        // assert_eq!(state.status, Status::Open);
    }
}

#[test]
fn test_create_proposal() {
    let mut deps = mock_dependencies();
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let voter1: Addr = app.api().addr_make("voter1");
    let count: u64 = 0;
    let proposals: BTreeMap<u64, Proposal> = BTreeMap::new();
    let info = message_info(&owner, &[]);
    setup_test_case(deps.as_mut(), info, count, proposals).unwrap();
    let bank_msg = BankMsg::Send {
        to_address: owner.into(),
        amount: vec![coin(1, "ucosm")],
    };
    let msgs = vec![CosmosMsg::Bank(bank_msg)];
    let env = mock_env();
    let ts = Timestamp::from_nanos(env.block.time.nanos()); // Mock timestamp

    let proposal = ExecuteMsg::Propose {
        title: "Che pasta ti piace?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec![
            "Norma".to_string(),
            "Carbonara".to_string(),
            "Gricia".to_string(),
        ],
        expires: Expiration::AtTime(ts.plus_days(2)),
        msgs: msgs,
    };
    let info = message_info(&voter1, &[]);

    let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap();
    // Verify
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "propose")
            .add_attribute("sender", voter1)
            .add_attribute("proposal_id", 1.to_string())
            .add_attribute("status", "Open")
    );
}

/*
#[test]
fn test_vote_works() {

    let app = App::default();
    let msg = InstantiateMsg {
        count: 0,
        proposals: BTreeMap::new(),
    };
    let owner = app.api().addr_make("owner");
    let voter1: Addr = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut deps = mock_dependencies();
    let info = message_info(&owner, &[]);
    let start = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone())
    .expect("failed to instantiate");
 let env = mock_env();
    println!("Ora: {}", env.block.time);
    let ts = Timestamp::from_nanos(env.block.time.nanos()); // Mock timestamp
    println!("Timestamp attuale in nanosecondi: {}", ts);
    println!("Scade a : {}", Expiration::AtTime(ts.plus_days(2)));

    let info = message_info(&owner, &[]);
    let msg = InstantiateMsg {
        title: "Che pasta ti piace?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec![
            "Norma".to_string(),
            "Carbonara".to_string(),
            "Gricia".to_string(),
        ],
        expiration: Expiration::AtTime(ts.plus_days(2)), // Expires in 2 days
    };



    let a_vote = ExecuteMsg::Vote { vote: Vote::A };

    let b_vote = ExecuteMsg::Vote { vote: Vote::B };
    let res = execute(deps.as_mut(), mock_env(), info, a_vote.clone()).unwrap();

    let info = message_info(&owner, &[]);

    let err = execute(deps.as_mut(), mock_env(), info.clone(), b_vote.clone()).unwrap_err();
    assert_eq!(err, ContractError::AlreadyVoted {});
    // Verify
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "vote")
            .add_attribute("sender", owner.clone())
            .add_attribute("status", "Open")
    );
    let info = message_info(&voter1, &[]);
    let ask_1 = ExecuteMsg::UpdateVoters {
        ask: info.sender.to_string(),
        add: [].to_vec(),
    };

    let response = execute(deps.as_mut(), mock_env(), info.clone(), ask_1.clone()).unwrap();

    let info = message_info(&voter1, &[]);

    let info_owner = message_info(&owner, &[]);
    let new_add_1 = ExecuteMsg::UpdateVoters {
        ask: "".to_string(),
        add: vec![voter1.to_string()],
    };
    let response = execute(deps.as_mut(), mock_env(), info_owner, new_add_1.clone()).unwrap();

    let vote1 = execute(deps.as_mut(), mock_env(), info, b_vote.clone()).unwrap();
    let vote = QueryMsg::Vote {
        voter: voter1.to_string(),
    };
    // println!("Il voto1 è: {:?}", vote1);
    println!("Il voto è: {:?}", vote);

    for item in BALLOTS.range(&deps.storage, None, None, Order::Ascending) {
        match item {
            Ok((voter, vote)) => {
                println!("Voter: {:?}, Vote: {:?}", voter, vote);
            }
            Err(err) => {
                // Gestisci l'errore
                eprintln!("Errore durante l'iterazione: {}", err);
            }
        }
    }
    let res = QueryMsg::Vote {
        voter: owner.to_string(),
    };
    let query_result: VoteResponse =
        from_json(query(deps.as_ref(), env.clone(), res).unwrap()).unwrap();
    println!("QueryMsg::Vote: {:?}", query_result);

    let vote_info = query_vote(deps.as_ref(), owner.to_string());
    println!("query_vote: {:?}", vote_info.unwrap().vote);
    let res = QueryMsg::Total {};
    let query_result: Votes = from_json(query(deps.as_ref(), env.clone(), res).unwrap()).unwrap();
    let allvote = query_proposal_response(deps.as_ref());
    println!("QueryMsg::Total: {:?}", query_result);
    println!("Query diretta: {:?}", allvote.unwrap());

    let res = QueryMsg::GetAllVotes {};
    let query_result: Vec<VoteInfo> = from_json(query(deps.as_ref(), env, res).unwrap()).unwrap();
    println!("QueryMsg::Total: {:?}", query_result);
}
*/

/*

    #[test]
    fn test_vote_request() {
        let app = App::default();

        let owner = app.api().addr_make("owner");
        let voter1: Addr = app.api().addr_make("voter1");
        let voter2 = app.api().addr_make("voter2");
        let mut deps = mock_dependencies();

        let env = mock_env();
        // println!("Ora: {}", env.block.time);
        let ts = Timestamp::from_nanos(env.block.time.nanos()); // Mock timestamp
        let info = message_info(&owner, &[]);
        let msg = InstantiateMsg {
            title: "Che pasta ti piace?".to_string(),
            description: "Dicci la tua preferenza!".to_string(),
            option: vec![
                "Norma".to_string(),
                "Carbonara".to_string(),
                "Gricia".to_string(),
            ],
            expiration: Expiration::AtTime(ts.plus_days(2)), // Expires in 2 days
        };

        let start = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone())
            .expect("failed to instantiate");

        let a_vote = ExecuteMsg::Vote { vote: Vote::A };

        let b_vote = ExecuteMsg::Vote { vote: Vote::B };
        let res = execute(deps.as_mut(), mock_env(), info, a_vote.clone()).unwrap();
        // Verify
        assert_eq!(
            res,
            Response::new()
                .add_attribute("action", "vote")
                .add_attribute("sender", owner.clone())
                .add_attribute("status", "Open")
        );
        let info = message_info(&voter1, &[]);
        let ask_1 = ExecuteMsg::UpdateVoters {
            ask: info.sender.to_string(),
            add: [].to_vec(),
        };

        let response = execute(deps.as_mut(), mock_env(), info.clone(), ask_1.clone()).unwrap();
        let err = execute(deps.as_mut(), mock_env(), info, a_vote.clone()).unwrap_err();
        // Verify: this account is unauthorized
        assert_eq!(err, ContractError::Unauthorized {});
        let info = message_info(&voter1, &[]);
        let add_1 = ExecuteMsg::UpdateVoters {
            ask: "".to_string(),
            add: vec![info.sender.to_string()],
        };
        let err = execute(deps.as_mut(), mock_env(), info, add_1.clone()).unwrap_err();
        // Verify: this account can't add someone to vote, need Check
        assert_eq!(err, ContractError::Unauthorized {});

        let info = message_info(&voter1, &[]);
        let info_owner = message_info(&owner, &[]);
        let new_add_1 = ExecuteMsg::UpdateVoters {
            ask: "".to_string(),
            add: vec![voter1.to_string()],
        };
        let response = execute(deps.as_mut(), mock_env(), info_owner, new_add_1.clone()).unwrap();

        let b_vote = ExecuteMsg::Vote { vote: Vote::B };
        let res = execute(deps.as_mut(), mock_env(), info, b_vote.clone()).unwrap();
        // Verify
        assert_eq!(
            res,
            Response::new()
                .add_attribute("action", "vote")
                .add_attribute("sender", voter1.clone())
                .add_attribute("status", "Open")
        );

        // Info voters
        for item in VOTERS.range(&deps.storage, None, None, Order::Ascending) {
            match item {
                Ok((voter, vote)) => {
                    println!("Addr: {:?}, Value: {:?}", voter, vote);
                }
                Err(err) => {
                    // Gestisci l'errore
                    eprintln!("Errore durante l'iterazione: {}", err);
                }
            }
        }
    }
}*/
