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
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    // #[error("Not an admin!")]
    // AdminError(AdminError)
}