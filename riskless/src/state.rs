use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Map, Item};
use cw_controllers::{Admin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProjectStatus {
    FundingInProgress,
    TargetMet,
    ProjectOffTrack
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Project {
    pub creator: Addr,
    pub project_status: ProjectStatus,
    pub target_principal_amount: Uint128,
    pub target_yield_amount: Uint128,
    pub principal_amount: Uint128,
    pub project_deadline: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub anchor_earn_contract_address: String
}

/*
    Key: 1234
    Value: { 
        FundingStatus::Open
        1_000_000_000 ($1,000 UST)
        1661402594 (Thursday, August 25, 2022 4:43:14 AM)
        0
        0
        "terra1..."
    }
}
*/
pub const PROJECTS: Map<&[u8], Project> = Map::new("projects");

/*
    Key: ("terra1...", 1)
    Value: {
        500_000_000 ($500 UST)
    }
*/
pub const BALANCES: Map<(&Addr, &[u8]), Uint128> = Map::new("balances");

/*
    Store contact admin
*/
pub const ADMIN: Admin = Admin::new("admin");

/*
    Config
*/
pub const CONFIG: Item<Config> = Item::new("config");
