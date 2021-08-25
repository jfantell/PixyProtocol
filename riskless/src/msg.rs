use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Timestamp, Addr};
use crate::state::{FundingStatus, Project};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateProject { id: String, name: String, target_amount: u32, fund_deadline: Timestamp, project_deadline : Timestamp },
    FundProject { id: String, amount: u32 },
    WithdrawProjectFunds { id: String, amount: u32 },
    ChangeProjectFundingStatus { id: String, fund_status: FundingStatus},
    DepositProjectFundsToAnchor { id: String },
    WithdrawProjectFundsFromAnchor { id: String } ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectStatus { project_id : String },
    GetUserBalance { project_id : String, user : Addr, },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProjectStatusResponse {
    pub project_status: Project,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserBalanceResponse {
    pub project_balance: u32,
}
