use cosmwasm_std::{to_binary, StdResult, CosmosMsg, WasmMsg, Coin, DepsMut, Uint128};
use cosmwasm_bignumber::{Decimal256, Uint256};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cw20::{Cw20ExecuteMsg};
use terra_cosmwasm::TerraQuerier;

const PLACEHOLDER_ADDRESS : &str = "";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    DepositStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

// Tax related functions
pub fn query_tax_rate(
    deps: DepsMut,
) -> StdResult<Decimal256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    Ok(terra_querier.query_tax_rate()?.rate.into())
}

pub fn compute_tax(
    deps: DepsMut,
    coin: &Coin,
) -> StdResult<Uint256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(
        amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax_rate)),
        tax_cap,
    ))
}

pub fn deduct_tax(deps: DepsMut, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (Uint256::from(coin.amount) - tax_amount).into(),
    })
}

// Deposit into Anchor
pub fn deposit_stable_msg(deps: DepsMut, amount: Uint128, denom: String) -> StdResult<CosmosMsg> {

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: PLACEHOLDER_ADDRESS.to_string(),
        msg: to_binary(&ExecuteMsg::DepositStable {})?,
        funds: vec![deduct_tax(
            deps,
            Coin {
                denom,
                amount,
            },
        )?],
    }))
}
// Widthdraw from Anchor
pub fn redeem_stable_msg(_deps: &DepsMut, amount: Uint128, _denom: String) -> StdResult<CosmosMsg> {
    let anchor_earn_contract_address = PLACEHOLDER_ADDRESS.to_string();
    let a_ust_address = PLACEHOLDER_ADDRESS.to_string();
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: a_ust_address,
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: anchor_earn_contract_address.clone(),
            amount: amount,
            msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        })?,
        funds: vec![],
    }))
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

// Compute aUST/UST excange rate
// pub fn epoch_state(deps: DepsMut, market: &CanonicalAddr) -> StdResult<EpochStateResponse> {
//     let epoch_state: EpochStateResponse =
//         deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//             contract_addr: deps.api.human_address(market)?,
//             msg: to_binary(&QueryMsg::EpochState {
//                 block_height: None,
//                 distributed_interest: None,
//             })?,
//         }))?;

//     Ok(epoch_state)
// }
