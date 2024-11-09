use std::default;

use cosmwasm_std::{
    coins,
    testing::{mock_dependencies, mock_env},
    Addr, Coin, Timestamp, Uint128,
};
use cw_multi_test::App;
use cw_utils::Expiration;

use crate::{
    msg::{ExecuteMsg, ProposalResponse, QueryMsg},
    state::{Proposal, State, Vote, Votes, STATUS},
    ContractError,
};

use super::contract::ElectwasmContract;

const UJUNO: &str = "ujunox";
const UATOM: &str = "uatom";

#[test]
fn test_instantiate_with_valid_message() {
    let owner = Addr::unchecked("owner"); // it won't works
    let mut app = App::default();
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

    // Asserto per verificare lo stato salvato
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
    let app = App::default();
    let owner = app.api().addr_make("owner"); // it won't works
    let proposer1 = app.api().addr_make("proposer1"); // it won't works
                                                      // let owner = Addr::unchecked("owner"); // it won't works
                                                      // let proposer1 = Addr::unchecked("proposer1"); // it won't works
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(500_000, UATOM))
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

    /* Try to create a proposal */
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

    // Not supported Token
    // let info = message_info(&voter1, &coins(1_000, "ucosm"));
    // let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap_err();
    // Verify
    // assert_eq!(res, ContractError::UnsupportedToken {});

    assert_eq!(
        err,
        ContractError::InsufficientFunds {
            funds: Uint128::new(400_000),
            commission: Uint128::new(500_000)
        }
    );

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(500_000, "uatom")
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
fn create_proposal_unsupported_funds() {
    let app = App::default();
    let owner = app.api().addr_make("owner"); // it won't works
    let proposer1 = app.api().addr_make("proposer1"); // it won't works
                                                      // let owner = Addr::unchecked("owner"); // it won't works
                                                      // let proposer1 = Addr::unchecked("proposer1"); // it won't works
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(500_000, UATOM))
            .unwrap();
    });
    let code_id = ElectwasmContract::store_code(&mut app);
    let commissions = vec![
        // Coin {
        //     denom: UATOM.to_string(),
        //     amount: Uint128::new(500_000),
        // },
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

    /* Try to create a proposal */
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
    // let info = message_info(&voter1, &coins(1_000, "ucosm"));
    // let res = execute(deps.as_mut(), mock_env(), info, proposal.clone()).unwrap_err();
    // Verify
    assert_eq!(err, ContractError::UnsupportedToken {});

    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(500_000, "uatom")
    );
}

#[test]
fn create_proposal_works() {
    let app = App::default();
    let owner = app.api().addr_make("owner"); // it won't works
    let proposer1 = app.api().addr_make("proposer1"); // it won't works
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &proposer1, coins(600_000, UATOM))
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

    /* Try to create a proposal */
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

    
    assert_eq!(
        app.wrap().query_all_balances(proposer1.clone()).unwrap(),
        coins(100_000, "uatom")
    );
}
