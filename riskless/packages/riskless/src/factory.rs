use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Timestamp, Uint128, Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub project_code_id : u64,
    pub anchor_money_market_address: Option<String>,
    pub a_ust_address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateProject { name: String, target_principal_amount: Uint128, target_yield_amount: Uint128, project_deadline: Timestamp },
    UpdateAdmin { new_admin: Option<String> } ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProjectContractAddress { name : String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProjectContractAddressResponse {
    pub address: Addr,
}