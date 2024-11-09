use cosmwasm_std::{coins, testing::mock_dependencies, Addr, Coin, Uint128};
use cw_multi_test::App;

use crate::{
    msg::{ProposalResponse, QueryMsg},
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
    let contract =
        ElectwasmContract::instantiate(&mut app, code_id, &owner, "First Election", commissions.clone(), 0)
            .unwrap();
    let state = STATUS.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Asserto per verificare lo stato salvato
    assert_eq!(
        state,
        State {
            admin:owner,
            commissions,
            voting_fee:0
        }
    )
}
