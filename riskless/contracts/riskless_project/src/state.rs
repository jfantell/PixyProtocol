use riskless::project::{Project};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Map, Item};
use cw_controllers::{Admin};
/*
    Key: ("terra1...", "film1")
    Value: {
        500_000_000 ($500 UST)
    }
*/
pub const BACKINGS: Map<&Addr, Uint128> = Map::new("backings");

pub const PROJECT: Item<Project> = Item::new("project");

pub const ADMIN: Admin = Admin::new("admin");