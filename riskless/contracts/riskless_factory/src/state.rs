use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Map, Item};
use cw_controllers::{Admin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub anchor_money_market_address: String,
    pub a_ust_address: String,
    pub project_code_id: u64,
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
pub const PROJECTS: Map<&[u8], Addr> = Map::new("projects");

/*
    Store contact admin
*/
pub const ADMIN: Admin = Admin::new("admin");

/*
    Config
*/
pub const CONFIG: Item<Config> = Item::new("config");

/*

*/
pub const TMP_PROJECT_NAME: Item<String> = Item::new("tmpprojectname");
