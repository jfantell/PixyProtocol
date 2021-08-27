use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Timestamp, Uint128};
use crate::state::{FundingStatus, Project};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateProject { id: String, name: String, target_amount: Uint128, fund_deadline: Timestamp, project_deadline : Timestamp },
    UpdateAdmin { new_admin: Option<String> } ,
    FundProject { id: String },
    WithdrawProjectFunds { id: String, amount: Uint128 },
    ChangeProjectFundingStatus { id: String, fund_status: FundingStatus},
    DepositProjectFundsToAnchor { id: String },
    WithdrawProjectFundsFromAnchor { id: String } ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectStatus { project_id : String },
    GetUserBalance { project_id : String, user: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProjectStatusResponse {
    pub project_status: Project,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserBalanceResponse {
    pub user_balance: Uint128,
}
