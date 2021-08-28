#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Timestamp, Uint128};

use crate::error::ContractError;
use crate::msg::{UserBalanceResponse, ProjectStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{FundingStatus, Project, PROJECTS, BALANCES};
use crate::state::{ADMIN};
use cw0::{maybe_addr};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    ADMIN.set(deps.branch(), maybe_addr(api, msg.admin)?)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => {
            let api = deps.api;
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, new_admin)?)?)
        },
        ExecuteMsg::CreateProject {
            id,
            name,
            target_amount,
            fund_deadline,
            project_deadline,
        } => try_create_project(deps, info, id, name, target_amount, fund_deadline, project_deadline),
        ExecuteMsg::FundProject {
            id,
        } => try_fund_project(deps, info, id),
        ExecuteMsg::WithdrawProjectFunds {
            id,
            amount
        } => try_widthdraw_project_funds(deps, info, id, amount),
        ExecuteMsg::ChangeProjectFundingStatus {
            id,
            fund_status
        } => try_change_project_funding_status(deps, info, id, fund_status),
        ExecuteMsg::DepositProjectFundsToAnchor {
            id
        } => try_deposit_project_funds_to_anchor( deps, info, id ),
        ExecuteMsg::WithdrawProjectFundsFromAnchor {
            id
        } => try_widthdraw_project_funds_to_anchor( deps, info, id )
    }
}

pub fn try_create_project(deps: DepsMut, info: MessageInfo, id: String, name: String,
    target_amount: Uint128, fund_deadline: Timestamp, project_deadline: Timestamp) -> Result<Response, ContractError> {
    let project = Project {
        name: name,
        creator: info.sender,
        fund_status: FundingStatus::Open,
        fund_target_amount: target_amount,
        fund_deadline: fund_deadline,
        project_deadline: project_deadline,
        fund_amount: Uint128::zero(),
        fund_yield_amount: Uint128::zero(),
    };

    // Save new project
    PROJECTS.save(deps.storage, id.as_bytes(), &project)?;

    Ok(Response::new()
        .add_attribute("method", "create_project")
        .add_attribute("project_id", id)
        .add_attribute("fund_target_uusd", target_amount.to_string()))
}
pub fn try_fund_project(deps: DepsMut, info: MessageInfo, id: String) -> Result<Response, ContractError> {
    let mut amount = Uint128::zero();
    for coin in info.funds {
        if coin.denom.eq("uusd")
        {
            amount = coin.amount;
            BALANCES.update(deps.storage, (&info.sender, id.as_bytes()), | current_amount | -> Result<_, ContractError> {
                Ok(current_amount.unwrap_or_default() + amount)
            })?;            
            break;
        }
    }
    
    Ok(Response::new()
        .add_attribute("action", "fund_project")
        .add_attribute("project_id", id)
        .add_attribute("amount_uusd", amount.to_string())
        .add_attribute("sender", info.sender))
}

pub fn try_widthdraw_project_funds(deps: DepsMut, info: MessageInfo, id: String, amount: Uint128) -> Result<Response, ContractError> {
    BALANCES.update(deps.storage, (&info.sender, id.as_bytes()), | current_amount | -> Result<_, ContractError> {
        Ok(current_amount.unwrap_or_default() - amount)
    })?;
    
    Ok(Response::default())
}

pub fn try_change_project_funding_status(deps: DepsMut, info: MessageInfo, id: String, fund_status: FundingStatus) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn try_deposit_project_funds_to_anchor(deps: DepsMut, info: MessageInfo, id: String) -> Result<Response, ContractError> {
    if ADMIN.is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::default())
}

pub fn try_widthdraw_project_funds_to_anchor(deps: DepsMut, info: MessageInfo, id: String) -> Result<Response, ContractError> {
    if ADMIN.is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectStatus { project_id } => to_binary(&query_project_status(deps, project_id )?),
        QueryMsg::GetUserBalance { project_id, user } => to_binary(&query_user_balance(deps, project_id, user)?),
    }
}

fn query_project_status(deps: Deps, project_id : String ) -> StdResult<ProjectStatusResponse> {
    let state = PROJECTS.load(deps.storage, project_id.as_bytes())?;
    Ok(ProjectStatusResponse { project_status: state })
}

fn query_user_balance(deps: Deps, project_id : String, user: Option<String>) -> StdResult<UserBalanceResponse> {
    let user_addr = maybe_addr(deps.api, user)?;
    // TO-DO Remove unwrap()
    let balance = BALANCES.load(deps.storage, (&user_addr.unwrap(), project_id.as_bytes()))?;
    Ok(UserBalanceResponse { user_balance: balance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    const TOKEN_MULTIPLIER : u128 = 1_000_000;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let sender = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v";

        // Initialize smart contract
        let msg = InstantiateMsg { admin: Some(sender.to_string()) };
        let info = mock_info(sender, &coins(0, "uusd"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        let target_amount = Uint128::from(1_000_000 * TOKEN_MULTIPLIER);

        // Create new project with id: film1
        let project_id = "film1".to_string();
        let msg = ExecuteMsg::CreateProject {
            id: project_id.clone(),
            name: project_id.clone(),
            target_amount: target_amount,
            fund_deadline : Timestamp::from_seconds(1630349707),
            project_deadline : Timestamp::from_seconds(1633028107)
        };
        let info = mock_info(sender, &coins(0, "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query status of recently created project with id: film1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProjectStatus { project_id : project_id.clone() }).unwrap();
        let value: ProjectStatusResponse = from_binary(&res).unwrap();
        let expected_fund_deadline = Timestamp::from_seconds(1630349707);
        assert_eq!(expected_fund_deadline, value.project_status.fund_deadline);

        // Fund project with 1000 uusd
        let fund_project_uusd = Uint128::from(1_000 * TOKEN_MULTIPLIER);
        let msg = ExecuteMsg::FundProject {
            id: project_id.clone(),
        };
        let info = mock_info(sender, &coins(fund_project_uusd.u128(), "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query user balance
        let msg = ExecuteMsg::FundProject {
            id: project_id.clone(),
        };
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUserBalance { project_id : project_id.clone(), user: Some(sender.to_string())}, ).unwrap();
        let value: UserBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(fund_project_uusd, value.user_balance);
    }
}
