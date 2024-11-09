use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already voted on this proposal")]
    AlreadyVoted {},

    #[error("Proposal voting period has expired")]
    Expired {},

    #[error("No vote")]
    NoVote {},
    #[error("Missing accept token")]
    MissingAcceptedToken {},
    #[error("Missing payment")]
    MissingPayment {},
    #[error("Proposal is NOT Open")]
    NotOpen {},
    #[error("Coin NOT Supported")]
    UnsupportedToken {},
    #[error("Invalid: Amount of token sent ({funds}) are lower than commission ({commission})")]
    InsufficientFunds {funds:Uint128, commission:Uint128},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
