use cosmwasm_std::StdError;
use thiserror::Error;
use cw_controllers::{AdminError};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Must deposit at least $20 UST")]
    DepositMinimumError {},

    #[error("Invalid address")]
    InvalidAddress {},

    #[error("Cannot withdraw funds")]
    CannotWidthdrawFunds {},

    #[error("Unable to fund project")]
    UnableToFundProject {},

    #[error("Unable to acquire yield")]
    UnableToAcquireYield,

    #[error("Unable to update contract state")]
    UnableToUpdateContractState
}