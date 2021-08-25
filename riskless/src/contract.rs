#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Timestamp, Addr};

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
    // Set contract admin
    let admin_addr = maybe_addr(deps.api, msg.admin)?;
    ADMIN.set(deps.branch(), admin_addr)?;

    // TO-DO: Remove force unwrap
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
        ExecuteMsg::CreateProject {
            id,
            name,
            target_amount,
            fund_deadline,
            project_deadline,
        } => try_create_project(deps, info, id, name, target_amount, fund_deadline, project_deadline),
        ExecuteMsg::FundProject { 
            id,
            amount 
        } => try_fund_project(deps, info, id, amount),
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
    target_amount: u32, fund_deadline: Timestamp, project_deadline: Timestamp) -> Result<Response, ContractError> {
    let project = Project {
        name: name,
        creator: info.sender,
        fund_status: FundingStatus::Open,
        fund_target_amount: target_amount,
        fund_deadline: fund_deadline,
        project_deadline: project_deadline,
        fund_amount: 0,
        fund_yield_amount: 0,
    };

    // Save new project
    PROJECTS.save(deps.storage, id.as_bytes(), &project)?;

    Ok(Response::default())
}
pub fn try_fund_project(deps: DepsMut, info: MessageInfo, id: String, amount: u32) -> Result<Response, ContractError> {
    BALANCES.update(deps.storage, (&info.sender, id.as_bytes()), | currentAmount | -> Result<_, ContractError> {
        Ok(currentAmount.unwrap_or_default() + amount)
    })?;
    Ok(Response::default())
}

pub fn try_widthdraw_project_funds(deps: DepsMut, info: MessageInfo, id: String, amount: u32) -> Result<Response, ContractError> {
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
        QueryMsg::GetProjectStatus { project_id } => to_binary(&query_project_status(deps, project_id)?),
        QueryMsg::GetUserBalance { project_id, user } => to_binary(&query_user_balance(deps, project_id, user)?),
    }
}

fn query_project_status(deps: Deps, project_id : String) -> StdResult<ProjectStatusResponse> {
    let state = PROJECTS.load(deps.storage, project_id.as_bytes())?;
    Ok(ProjectStatusResponse { project_status: state })
}

fn query_user_balance(deps: Deps, project_id : String, user: Addr) -> StdResult<UserBalanceResponse> {
    let balance = BALANCES.load(deps.storage, (&user, project_id.as_bytes()))?;
    Ok(UserBalanceResponse { project_balance: balance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { admin: Some("terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string()) };
        let info = mock_info("terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v", &coins(1000, "Riskless"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let project_id = "film1".to_string();

        let msg = ExecuteMsg::CreateProject {
            id: project_id.clone(),
            name: project_id.clone(),
            target_amount: 1_000_000_000,
            fund_deadline : Timestamp::from_seconds(1630349707),
            project_deadline : Timestamp::from_seconds(1633028107)
        };
        let info = mock_info("terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v", &coins(1000, "Riskless"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProjectStatus { project_id : project_id.clone() }).unwrap();
        let value: ProjectStatusResponse = from_binary(&res).unwrap();
        let expected_fund_deadline = Timestamp::from_seconds(1630349707);
        assert_eq!(expected_fund_deadline, value.project_status.fund_deadline);
    }
}
