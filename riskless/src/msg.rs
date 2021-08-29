use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Timestamp, Uint128};
use crate::state::{ProjectStatus, Project};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub anchor_earn_contract_address: Option<String> 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateProject { name: String, target_principal_amount: Uint128, target_yield_amount: Uint128, project_deadline: Timestamp },
    UpdateAdmin { new_admin: Option<String> } ,
    FundProject { name: String },
    WidthdrawPrincipal { name: String },
    ChangeProjectStatus { name: String, project_status: ProjectStatus },
    WidthdrawYield { name: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectStatus { name : String },
    GetUserBalance { name : String, user: Option<String> },
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
