use cosmwasm_std::{coins, testing::mock_env, Addr, Coin, Timestamp, Uint128};
use cw_multi_test::App;
use cw_utils::Expiration;

use crate::{
    msg::ExecuteMsg,
    state::{State, STATUS, VOTERS},
    ContractError,
};

use super::contract::ElectwasmContract;
/* DEFINE CONSTANT and utility functions  */
const UJUNO: &str = "ujunox";
const UATOM: &str = "uatom";

pub fn advance_time(app: &mut App, seconds: u64) {
    app.update_block(|block| {
        block.time = block.time.plus_seconds(seconds);
    });
}

#[test]
fn test_instantiate_with_valid_message() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();
    let state = STATUS.query(&app.wrap(), contract.addr().clone()).unwrap();
    println!("admin {:?}", state.admin);
    println!("code_id {:?}", code_id);

    // Assertion to check the saved state
    assert_eq!(
        state,
        State {
            admin: owner,
            commissions,
            voting_fee: 0
        }
    )
}

#[test]
fn create_proposal_insufficient_funds() {
    /* Define Utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(500_000, UATOM))
            .unwrap();
    });
    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();
    /* End Instantiate */
    /* Start create a proposal */
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
    };
    let err = contract
        .create_proposal(&mut app, &proposer1, &coins(400_000, UATOM), proposal)
        .unwrap_err();
    assert_eq!(
        err,
        ContractError::InsufficientFunds {
            funds: Uint128::new(400_000),
            commission: Uint128::new(500_000)
        }
    );
    /* Verify correct balance */
    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(500_000, "uatom")
    );
}

#[test]
fn create_proposal_unsupported_funds() {
    /* Define Utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(500_000, UATOM))
            .unwrap();
    });
    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![Coin {
        denom: UJUNO.to_string(),
        amount: Uint128::new(500_000),
    }];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();
    /* End Instantiate */
    /* Start create a proposal */
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
    };
    let err = contract
        .create_proposal(&mut app, &proposer1, &coins(400_000, "uatom"), proposal)
        .unwrap_err();

    // Not supported Token
    assert_eq!(err, ContractError::UnsupportedToken {});

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(500_000, "uatom")
    );
}

#[test]
fn create_proposal_works() {
    /* Define Utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });
    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();
    /* End Instantiate */
    /* Start create a proposal */
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
    };
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    /* Verify new balances */

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(100_000, "uatom")
    );
    assert_eq!(
        app.wrap().query_all_balances(owner.clone()).unwrap(),
        coins(500_000, "uatom")
    );
}

#[test]
fn create_multiple_proposals() {
    /* Define Utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(1200_000, UATOM))
            .unwrap();
    });
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* Start create a proposal */
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
    };
    let proposal_2 = ExecuteMsg::Propose {
        title: "Ti piacciono i giochi?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec!["Si".to_string(), "No".to_string(), "Boh".to_string()],
        expires: Expiration::AtTime(ts.plus_days(2)),
    };
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal_2)
        .unwrap();
    /* Verify new balances */
    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(200_000, "uatom")
    );
    assert_eq!(
        app.wrap().query_all_balances(owner.clone()).unwrap(),
        coins(1000_000, "uatom")
    );
}

#[test]
fn test_vote_request_unauthorized() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &proposer1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let err = contract
        .voters_action(&mut app, &voter2, add_action)
        .unwrap_err();
    /* Someone that is no the owner of the proposal can't accept */
    assert_eq!(err, ContractError::Unauthorized {});
}
#[test]
fn test_vote_request_from_owner() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Check if voter2 can vote now */
    let proposal = VOTERS
        .query(&app.wrap(), contract.addr().clone(), (proposal_id, &voter2))
        .unwrap();
    println!("proposer value {:?}", proposal);
}
/* Voting Test */
#[test]
fn test_voting_unauthorized() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let err = contract
        .vote_proposal(&mut app, &voter1, b_vote)
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}
#[test]
fn test_simple_vote() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let _response = contract.vote_proposal(&mut app, &voter2, b_vote).unwrap();
}
#[test]
fn test_double_vote() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    let _response = contract.vote_proposal(&mut app, &voter2, b_vote).unwrap();
    let err = contract
        .vote_proposal(&mut app, &voter2, a_vote)
        .unwrap_err();
    /* Someone that is no the owner of the proposal can't accept */
    assert_eq!(err, ContractError::AlreadyVoted {});
}

#[test]
fn test_vote_proposal_closed() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();
    /* End Instantiate */

    /* Start create a proposal */
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
        expires: Expiration::AtTime(ts),
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap_err();
    /* If closed no one che ask to join */
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let err = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap_err();
    assert_eq!(err, ContractError::Expired {});
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    let err = contract
        .vote_proposal(&mut app, &voter2, b_vote)
        .unwrap_err();
    assert_eq!(err, ContractError::Expired {});
    let err = contract
        .vote_proposal(&mut app, &voter2, a_vote)
        .unwrap_err();
    /* Someone that is no the owner of the proposal can't accept */
    assert_eq!(err, ContractError::Expired {});
}

/* Removing test */

#[test]
fn test_remove_voters_from_proposal() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter4, b_vote).unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start removing address to voters list */
    let rmv_action = ExecuteMsg::UpdateVoters {
        ask: "".to_string(),
        add: vec![],
        rmv: vec![voter1.to_string()],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &proposer1, rmv_action)
        .unwrap();
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    /* End removing address to voters list */
}

#[test]
fn test_remove_voters_from_proposal_cannot_remove() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter4, b_vote).unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start removing address to voters list */
    let rmv_action = ExecuteMsg::UpdateVoters {
        ask: "".to_string(),
        add: vec![],
        rmv: vec![voter5.to_string()],
        proposal_id,
    };

    let err = contract
        .voters_action(&mut app, &proposer1, rmv_action)
        .unwrap_err();
    assert_eq!(err, ContractError::CannotRemoveVoter {})
    /* End removing address to voters list */
}

#[test]
fn test_remove_voters_from_proposal_unauthorized() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter4, b_vote).unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start removing address to voters list */
    let rmv_action = ExecuteMsg::UpdateVoters {
        ask: "".to_string(),
        add: vec![],
        rmv: vec![voter1.to_string()],
        proposal_id,
    };

    let err = contract
        .voters_action(&mut app, &voter1, rmv_action)
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    /* End removing address to voters list */
}

/* Closing Test */

#[test]
fn test_close_proposal_unauthorized() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let _response = contract.vote_proposal(&mut app, &voter2, b_vote).unwrap();
    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 259200);
    let close = ExecuteMsg::Close { proposal_id };
    let err = contract
        .close_proposal(&mut app, &voter2, close)
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    /* Someone that is no the owner of the proposal can't accept */
}

#[test]
fn test_close_proposal() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };

    let _response = contract.vote_proposal(&mut app, &voter2, b_vote).unwrap();

    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 259200);
    let close = ExecuteMsg::Close { proposal_id };
    let _response = contract
        .close_proposal(&mut app, &proposer1, close)
        .unwrap();
    /* Someone that is no the owner of the proposal can't accept */
}
#[test]
fn test_close_proposal_before_end() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![voter2.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };

    let _response = contract.vote_proposal(&mut app, &voter2, b_vote).unwrap();

    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 86400);
    let close = ExecuteMsg::Close { proposal_id };
    let err = contract
        .close_proposal(&mut app, &proposer1, close)
        .unwrap_err();
    assert_eq!(err, ContractError::NotExpired {})
    /* Someone that is no the owner of the proposal can't accept */
}

/* Query test */

#[test]
fn test_query_proposal_info() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */

    /* Query proposal info */
    let query_response = contract.query_proposal_info(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
}
#[test]
fn test_query_proposal_running_response() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    let _c_vote = ExecuteMsg::Vote {
        vote: 2,
        proposal_id,
    };
    let _d_vote = ExecuteMsg::Vote {
        vote: 3,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter4, b_vote).unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 259200);
    let close = ExecuteMsg::Close { proposal_id };
    let _response = contract
        .close_proposal(&mut app, &proposer1, close)
        .unwrap();
}
#[test]
fn test_query_proposal_winner() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    // assert_eq!(query_response,);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    let _c_vote = ExecuteMsg::Vote {
        vote: 2,
        proposal_id,
    };
    let _d_vote = ExecuteMsg::Vote {
        vote: 3,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter4, b_vote).unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 259200);
    let close = ExecuteMsg::Close { proposal_id };
    let _response = contract
        .close_proposal(&mut app, &proposer1, close)
        .unwrap();
    let query_response = contract.query_proposal_winner(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
}
#[test]
fn test_query_proposal_no_winner() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    // assert_eq!(query_response,);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */

    /* Start try voting */
    let b_vote = ExecuteMsg::Vote {
        vote: 1,
        proposal_id,
    };
    let a_vote = ExecuteMsg::Vote {
        vote: 0,
        proposal_id,
    };
    contract
        .vote_proposal(&mut app, &voter2, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 2");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter3, b_vote.clone())
        .unwrap();
    println!("Ha votato voter 3");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract
        .vote_proposal(&mut app, &voter4, a_vote.clone())
        .unwrap();
    println!("Ha votato voter 4");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    contract.vote_proposal(&mut app, &voter5, a_vote).unwrap();
    println!("Ha votato voter 5");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);
    // Advance time by 1 day (86400 seconds)
    advance_time(&mut app, 259200);
    let close = ExecuteMsg::Close { proposal_id };
    let _response = contract
        .close_proposal(&mut app, &proposer1, close)
        .unwrap();
    let query_response = contract.query_proposal_winner(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
}

#[test]
fn query_multiple_proposals() {
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(1200_000, UATOM))
            .unwrap();
    });
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* Start create a proposal */
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
    };
    let proposal_2 = ExecuteMsg::Propose {
        title: "Ti piacciono i giochi?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec!["Si".to_string(), "No".to_string(), "Boh".to_string()],
        expires: Expiration::AtTime(ts.plus_days(2)),
    };
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal_2)
        .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(200_000, "uatom")
    );
    assert_eq!(
        app.wrap().query_all_balances(owner.clone()).unwrap(),
        coins(1000_000, "uatom")
    );

    let query_response = contract.query_all_proposals(&app).unwrap();
    println!("{:?}", query_response);
}

#[test]
fn query_proposals_by_proposer() {
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let proposer2 = app.api().addr_make("proposer2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(1200_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &proposer2, coins(600_000, UATOM))
            .unwrap();
    });
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* Start create a proposal */
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
    };
    let proposal_2 = ExecuteMsg::Propose {
        title: "Ti piacciono i giochi?".to_string(),
        description: "Dicci la tua preferenza!".to_string(),
        option: vec!["Si".to_string(), "No".to_string(), "Boh".to_string()],
        expires: Expiration::AtTime(ts.plus_days(2)),
    };
    let _resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();
    let _resp = contract
        .create_proposal(&mut app, &proposer2, &coins(500_000, UATOM), proposal_2)
        .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(700_000, "uatom")
    );
    assert_eq!(
        app.wrap().query_all_balances(proposer2.clone()).unwrap(),
        coins(100_000, "uatom")
    );
    assert_eq!(
        app.wrap().query_all_balances(owner.clone()).unwrap(),
        coins(1000_000, "uatom")
    );

    let query_response = contract
        .query_proposals_by_proposer(&app, proposer2)
        .unwrap();
    println!("{:?}", query_response);
}
#[test]
fn query_proposal_voters() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");
    let voter5 = app.api().addr_make("voter5");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
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
    };
    let resp = contract
        .create_proposal(&mut app, &proposer1, &coins(500_000, UATOM), proposal)
        .unwrap();

    // Extract ID from response attributes
    let proposal_id = resp
        .events
        .iter()
        .flat_map(|event| event.attributes.iter())
        .find(|attr| attr.key == "proposal_id")
        .expect("Proposal ID not found")
        .value
        .parse::<u64>()
        .expect("Failed to parse proposal ID");

    println!("Created proposal ID: {}", proposal_id);
    /*End create proposal */
    println!("Prima");
    let query_response = contract
        .query_proposal_running_response(&app, proposal_id)
        .unwrap();
    println!("{:?}", query_response);

    /* Start ask join to proposal */
    let ask_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![],
        rmv: vec![],
        proposal_id,
    };

    let _resp = contract
        .voters_action(&mut app, &voter1, ask_action)
        .unwrap();
    /* End ask join to proposal */

    /* Start try to accept to proposal */

    let add_action = ExecuteMsg::UpdateVoters {
        ask: voter1.to_string(),
        add: vec![
            voter2.to_string(),
            voter3.to_string(),
            voter4.to_string(),
            voter5.to_string(),
        ],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    /* Voter Added */
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
    let add_action = ExecuteMsg::UpdateVoters {
        ask: "".to_string(),
        add: vec![voter1.to_string()],
        rmv: vec![],
        proposal_id,
    };

    let _response = contract
        .voters_action(&mut app, &proposer1, add_action)
        .unwrap();
    let query_response = contract.query_proposal_voters(&app, proposal_id).unwrap();
    println!("{:?}", query_response);
}
#[test]
fn query_contract_status() {
    /* Define utilities */
    let app = App::default();
    let owner = app.api().addr_make("owner");
    let proposer1 = app.api().addr_make("proposer1");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
            .unwrap();
    });

    /* Start Instantiate */
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        Coin {
            denom: UATOM.to_string(),
            amount: Uint128::new(500_000),
        },
        Coin {
            denom: UJUNO.to_string(),
            amount: Uint128::new(500_000),
        },
    ];
    let contract = ElectwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "First Election",
        commissions.clone(),
        0,
    )
    .unwrap();

    /* End Instantiate */

    /* Start create a proposal */
    let query_response = contract.query_contract_status(&app).unwrap();
    println!("{:?}", query_response);
}
