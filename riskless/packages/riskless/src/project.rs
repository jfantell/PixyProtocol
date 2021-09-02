use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr, Timestamp};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProjectStatus {
    FundingInProgress,
    TargetMet,
    ProjectOffTrack
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Project {
    pub name: String,
    pub creator: Addr,
    pub creation_date: Timestamp,
    pub project_status: ProjectStatus,
    pub target_principal_amount: Uint128,
    pub target_yield_amount: Uint128,
    pub principal_amount: Uint128,
    pub project_deadline: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub project: Project,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateAdmin { new_admin: Option<String> } ,
    FundProject { },
    WidthdrawPrincipal { },
    ChangeProjectStatus { project_status: Option<ProjectStatus> },
    WidthdrawYield { },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectStatus { },
    GetUserBalance { user: Option<String> },
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
