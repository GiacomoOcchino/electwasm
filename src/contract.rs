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
        accepted_tokens: msg.accepted_tokens,
        proposal_commission: msg.proposal_commission,
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
    use ExecuteMsg::{Propose, UpdateVoters, Vote};
    match msg {
        Propose {
            title,
            description,
            option,
            expires,
            msgs,
        } => exec::execute_create_proposal(
            deps,
            env,
            info,
            title,
            description,
            option,
            expires,
            msgs,
        ),
        Vote { vote, proposal_id } => exec::execute_vote(deps, env, info, vote, proposal_id),
        UpdateVoters {
            add,
            ask,
            proposal_id,
        } => exec::execute_update_voters(deps, env, info, add, ask, proposal_id),
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

#[cfg(test)]
mod tests {
    use crate::state::{Proposal, Vote, Votes, STATUS};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        coin, coins, from_json, Addr, BankMsg, CosmosMsg, Empty, Timestamp, Uint128,
    };
    use cw_multi_test::App;
    use cw_utils::Expiration;
    use std::collections::BTreeMap;

    use super::*;
    use crate::msg::{InstantiateMsg, ProposalResponse};

    // this will set up the instantiation for other tests
    #[track_caller]
    fn setup_test_case(
        deps: DepsMut,
        info: MessageInfo,
        accepted_tokens: Vec<String>,
        proposal_commission: u128,
        voting_fee: u64,
    ) -> Result<Response<Empty>, ContractError> {
        // Instantiate a contract with voters
        let instantiate_msg = InstantiateMsg {
            accepted_tokens,
            proposal_commission,
            voting_fee,
        };
        instantiate(deps, mock_env(), info, instantiate_msg)
    }
    /*Done */
    #[test]
    fn test_instantiate_with_valid_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        // let ts = Timestamp::from_nanos(env.block.time.nanos()); // Mock timestamp
        let msg = InstantiateMsg {
            accepted_tokens: vec!["uatom".to_string(), "ujunox".to_string()],
            proposal_commission: 500_000,
            voting_fee: 0,
        };
        let app = App::default();

        let owner = app.api().addr_make("owner");

        let info = message_info(&owner, &[]);

        let response = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone())
            .expect("failed to instantiate");

        // Asserto per verificare la risposta
        assert_eq!(response, Response::default());

        // Asserto per verificare lo stato salvato
        let state = STATUS.load(&deps.storage).expect("failed to load state");
        assert_eq!(state.accepted_tokens, msg.accepted_tokens);
        assert_eq!(state.admin, info.sender);
        assert_eq!(state.proposal_commission, msg.proposal_commission);
    }

    #[test]
    fn create_proposal_insufficient_funds() {
        // define owner and proposer
        let app = App::default();
        let owner = app.api().addr_make("owner");
        let voter1: Addr = app.api().addr_make("voter1");

        let mut deps = mock_dependencies();

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &voter1, coins(400_000, "ujunox"))
                .unwrap();
        });

        let info = message_info(&owner, &[]);
        /*Start Istantiate */
        let accepted_tokens = vec!["ujunox".to_string(), "uatom".to_string()];
        let proposal_commission = 500_000;
        let voting_fee = 0;

        setup_test_case(
            deps.as_mut(),
            info,
            accepted_tokens,
            proposal_commission,
            voting_fee,
        )
        .unwrap();

        /*End Istantiate */

        /*Start create proposal */

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
        // Not supported Token
        let info = message_info(&voter1, &coins(1_000, "ucosm"));
        let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap_err();
        // Verify
        assert_eq!(res, ContractError::UnsupportedToken {});

        let info = message_info(&voter1, &coins(400_000, "ujunox"));
        let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap_err();

        assert_eq!(
            res,
            ContractError::InsufficientFunds {
                funds: Uint128::new(400_000),
                commission: Uint128::new(500_000)
            }
        );

        // assert_eq!(
        //     res,
        //     Response::new()
        //         .add_attribute("commission_payer", voter1.clone())
        //         .add_attribute("action", "propose")
        //         .add_attribute("sender", voter1)
        //         .add_attribute("proposal_id", 1.to_string())
        //         .add_attribute("status", "Open")
        // );
    }

    #[test]
    fn test_create_proposal() {
        let mut deps = mock_dependencies();
        let app = App::default();

        let owner = app.api().addr_make("owner");
        let voter1: Addr = app.api().addr_make("voter1");
        let info = message_info(&owner, &[]);
        /*Start Istantiate */
        let accepted_tokens = vec!["ujunox".to_string(), "uatom".to_string()];
        let proposal_commission = 500_000;
        let voting_fee = 0;

        setup_test_case(
            deps.as_mut(),
            info,
            accepted_tokens,
            proposal_commission,
            voting_fee,
        )
        .unwrap();

        /*End Istantiate */

        /*Start create proposal */

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

    #[test]
    fn test_vote_works() {
        // Instantiate contract
        let mut deps = mock_dependencies();
        let app = App::default();
        let owner = app.api().addr_make("owner");
        // let voter1: Addr = app.api().addr_make("voter1");
        let count: u64 = 0;
        let proposals: BTreeMap<u64, Proposal> = BTreeMap::new();
        let info = message_info(&owner.clone(), &[]);
        // setup_test_case(deps.as_mut(), info.clone(), count, proposals).unwrap();

        // Create propose
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
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
        let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap();

        // Get the proposal id from the logs
        let proposal_id: u64 = res.attributes[2].value.parse().unwrap();

        let voter1 = app.api().addr_make("voter1");
        let voter2 = app.api().addr_make("voter2");

        let info_test_vote_1 = message_info(&voter1, &[]);

        // test voter with no permission
        let a_vote = ExecuteMsg::Vote {
            proposal_id,
            vote: Vote::A,
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info_test_vote_1.clone(),
            a_vote.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // test ask to join works

        let info_ask_1 = message_info(&voter1, &[]);
        let ask_1 = ExecuteMsg::UpdateVoters {
            proposal_id,
            ask: info_ask_1.sender.to_string(),
            add: [].to_vec(),
        };
        let response =
            execute(deps.as_mut(), mock_env(), info_ask_1.clone(), ask_1.clone()).unwrap();
        // Verify
        assert_eq!(
            response,
            Response::new()
                .add_attribute("action", "update_voters")
                .add_attribute("proposal", 1.to_string())
                .add_attribute("added", 0.to_string())
                .add_attribute("ask to join", voter1.clone())
                .add_attribute("sender", voter1.clone())
        );
        // owner accept voter1 to join
        let info_owner = message_info(&owner, &[]);
        let add_1 = ExecuteMsg::UpdateVoters {
            proposal_id,
            ask: "".to_string(),
            add: vec![voter1.to_string()],
        };
        let response = execute(deps.as_mut(), mock_env(), info_owner, add_1.clone()).unwrap();
        // Verify
        assert_eq!(
            response,
            Response::new()
                .add_attribute("action", "update_voters")
                .add_attribute("proposal", 1.to_string())
                .add_attribute("added", 1.to_string())
                .add_attribute("ask to join", "")
                .add_attribute("sender", owner.clone())
        );

        // Test vote from voter1 works

        let response_vote_ok = execute(
            deps.as_mut(),
            mock_env(),
            info_test_vote_1.clone(),
            a_vote.clone(),
        )
        .unwrap();

        // Verify
        assert_eq!(
            response_vote_ok,
            Response::new()
                .add_attribute("action", "vote")
                .add_attribute("sender", voter1.clone())
                .add_attribute("proposal_id", 1.to_string())
                .add_attribute("status", "Open")
        );

        // Test voting twice Error
        let b_vote = ExecuteMsg::Vote {
            proposal_id,
            vote: Vote::B,
        };

        let err = execute(
            deps.as_mut(),
            mock_env(),
            info_test_vote_1.clone(),
            b_vote.clone(),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyVoted {});

        // Query proposal info
        let query_proposal = QueryMsg::Proposal { proposal_id };

        let query_proposal_true: ProposalResponse =
            from_json(query(deps.as_ref(), mock_env(), query_proposal).unwrap()).unwrap();

        println!("query_response_true diretta: {:?}", query_proposal_true);

        // Query proposal response
        let query_proposal_response = QueryMsg::Total { proposal_id };
        let query_proposal_response_true: Votes =
            from_json(query(deps.as_ref(), mock_env(), query_proposal_response).unwrap()).unwrap();
        println!(
            "query_proposal_response_true: {:?}",
            query_proposal_response_true
        );
    }

    /*
    let b_vote = ExecuteMsg::Vote {proposal_id, vote: Vote::B };

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
        proposal_id,
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
    println!("QueryMsg::Total: {:?}", query_result);*/
}
/*
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
