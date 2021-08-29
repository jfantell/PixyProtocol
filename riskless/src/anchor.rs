use cosmwasm_std::{to_binary, StdResult, CosmosMsg, WasmMsg, Coin, DepsMut, Uint128};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::state::{CONFIG};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    DepositStable {},
}

pub fn deposit_stable_msg(deps: &DepsMut, amount: Uint128, denom: String) -> StdResult<CosmosMsg> {

    let cfg = CONFIG.load(deps.storage)?;
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.anchor_earn_contract_address,
        msg: to_binary(&ExecuteMsg::DepositStable {})?,
        funds: vec![
            Coin {
                denom,
                amount,
            },
        ],
    }))
}


