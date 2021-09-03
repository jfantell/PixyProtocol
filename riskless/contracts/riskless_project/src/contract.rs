#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Uint128, StdError};

use cosmwasm_std::Decimal as CwDecimal;
use crate::error::ContractError;
use riskless::project::{ProjectStatus, UserBalanceResponse, ProjectStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, BACKINGS, PROJECT};

use cw0::{maybe_addr};

const BASE_AMOUNT : u128 = 1_000_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ADMIN.set(deps.branch(), Some(info.sender))?;
    PROJECT.save(deps.storage, &msg.project)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => {
            let api = deps.api;
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, new_admin)?)?)
        },
        ExecuteMsg::FundProject {
        } => fund_project(deps, info),
        ExecuteMsg::WithdrawPrincipal {
        } => withdraw_principal(deps, env, info),
        ExecuteMsg::ChangeProjectStatus {
            project_status
        } => change_status(deps, env, info, project_status),
        ExecuteMsg::WithdrawYield {
        } => withdraw_yield(deps, env, info) 
    }
}

pub fn fund_project(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let state =  PROJECT.load(deps.storage)?;
    // If project off track, prevent backers from funding
    if state.project_status == ProjectStatus::ProjectClosedFail || state.project_status == ProjectStatus::ProjectClosedSuccess {
        return Err(ContractError::UnableToFundProject {} )
    }

    // Extract amount of UST sent to contract
    let required_denom = "uusd".to_string();
    let deposit_amount: Uint128 = info.funds
        .iter()
        .find(|c| c.denom == required_denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);

    // Ensure amount is more than $20
    if deposit_amount < Uint128::from(20 * BASE_AMOUNT) {
        return Err(ContractError::DepositMinimumError {} )
    }

    // TO-DO: Deposit to Anchor

    // Update user balance
    BACKINGS.update(deps.storage, &info.sender, | current_amount | -> Result<_, ContractError> {
        Ok(current_amount.unwrap_or_default() + deposit_amount)
    })?;

    // Update project stats
    PROJECT.update(deps.storage, | mut project | -> Result<_, ContractError> { 
        project.principal_amount += deposit_amount;
        Ok(project)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "fund_project")
        .add_attribute("amount_uusd", deposit_amount.to_string())
        .add_attribute("sender", info.sender))
}

// Backer withdraws their principal.
pub fn withdraw_principal(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = PROJECT.load(deps.storage)?;
    let withdraw_amount = BACKINGS.load(deps.storage, &info.sender)?;
    let yield_ = match get_yield_amount(&deps, &env, &info) {
        Ok(y) => { y },
        Err(_) => { return Err(ContractError::UnableToAcquireYield {} ) }
    };
    
    // Backers cannot withdraw principal if the following is true:
    // project status is TargetMet && yield target has not been met
    if (state.project_status == ProjectStatus::Delivery) && ( yield_ < state.target_yield_amount ) {
        return Ok(Response::new()
            .add_attribute("action", "withdraw_principal")
            .add_attribute("status", "cannot withdraw principal: target funding met and yield less than target yield")
            .add_attribute("sender", info.sender));
    }
    
    BACKINGS.update(deps.storage, &info.sender, | current_amount | -> Result<_, ContractError> {
        Ok(current_amount.unwrap_or_default() - withdraw_amount)
    })?;
    
    PROJECT.update(deps.storage, | mut project | -> Result<_, ContractError> {
        project.principal_amount -= withdraw_amount;
        Ok(project)
    })?;

    Ok(Response::new()
        .add_attribute("action", "withdraw_principal")
        .add_attribute("amount_uusd", withdraw_amount.to_string())
        .add_attribute("sender", info.sender))
}

// Compounded interest equation: (initial * (1 + rate)^DAYS) - initial
pub fn compute_yield(start_amount: f32, number_of_days: f32) -> Uint128
{
    let daily_rate: f32 = 0.20 / 365.;
    let daily_rate_exp = (1. + daily_rate).powf(number_of_days);
    let yield_ = (start_amount * daily_rate_exp) - start_amount;
    Uint128::from(yield_ as u128)
}

pub fn get_yield_amount(deps: &DepsMut, env: &Env, _info: &MessageInfo) -> Result<Uint128, ContractError> {
    let state = PROJECT.load(deps.storage)?;
    let start_amount = state.principal_amount.u128() as f32;
    let creation_date = state.creation_date;
    let current_date = env.block.time;
    let number_of_days: f32 = (current_date.minus_seconds(creation_date.seconds()).seconds() / (60 * 60 * 24)) as f32;
    return Ok(compute_yield(start_amount, number_of_days));
}

pub fn change_status(deps: DepsMut, env: Env, info: MessageInfo, project_status: Option<ProjectStatus>) -> Result<Response, ContractError> {
    let mut state = PROJECT.load(deps.storage)?;

    // Admin can change project status at any time
    if ADMIN.is_admin(deps.as_ref(), &info.sender)? {
        match project_status.clone() {
            Some(status) => { state.project_status = status },
            None => {}
        }
    }

    // Fund deadline and principal amount target has been met, change status to TargetMet
    // users can no longer Withdraw principal
    if env.block.time >= state.fund_deadline && state.project_status == ProjectStatus::FundingInProgress {
        if state.principal_amount >= state.target_principal_amount {
            state.project_status = ProjectStatus::Delivery;
        }
        else {
            state.project_status = ProjectStatus::ProjectClosedFail;
        }
    }

    PROJECT.save(deps.storage, &state)?;
    
    return Ok(Response::new()
        .add_attribute("action", "change_status")
        .add_attribute("status", format!("{:?}", project_status.clone()))
        .add_attribute("sender", info.sender));
}

pub fn withdraw_yield(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = PROJECT.load(deps.storage)?;

    let yield_ = match get_yield_amount(&deps, &env, &info) {
        Ok(y) => { y },
        Err(_) => { return Err(ContractError::UnableToAcquireYield {} ) }
    };

    // Creator withdraws yield
    if info.sender == state.creator  {
        if state.target_yield_amount >= get_yield_amount(&deps, &env, &info)? && state.project_status != ProjectStatus::ProjectClosedFail {
            if state.project_status != ProjectStatus::ProjectClosedSuccess {
                return Err(ContractError::CreatorUnableToWithdrawYield {}); 
            }
            else {
                return Ok(Response::new()
                    .add_attribute("action", "withdraw_yield")
                    .add_attribute("status", "withdrew all yield")
                    .add_attribute("amount", yield_.to_string())
                    .add_attribute("sender", info.sender))
            }
        }
        else {
            return Ok(Response::new()
                .add_attribute("action", "withdraw_yield")
                .add_attribute("status", "cannot withdraw yield: project off track")
                .add_attribute("sender", info.sender))
        }
    }
    // Financial backer withdraws yield (only can do this upon project completion or if project fails)
    else {
        let backer_principal = BACKINGS.load(deps.storage, &info.sender)?;
        let backer_ratio = CwDecimal::from_ratio(backer_principal, state.principal_amount);
        let backer_yield = backer_ratio * yield_;
    
        // Return error if target yield not met or the project failed
        if yield_ <= state.target_yield_amount && state.project_status != ProjectStatus::ProjectClosedFail {
            return Ok(Response::new()
                .add_attribute("action", "withdraw_yield")
                .add_attribute("status", "cannot withdraw yield: target yield has not been met")
                .add_attribute("sender", info.sender)
            )
        }

        // Calculate yield and subtract from k
        BACKINGS.update(deps.storage, &info.sender, | current_amount | -> Result<_, ContractError> {
            Ok(current_amount.unwrap_or_default() - backer_yield)
        })?;
        
        Ok(Response::new()
            .add_attribute("action", "withdraw_yield")
            .add_attribute("sender", info.sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectStatus { } => to_binary(&query_project_status(deps )?),
        QueryMsg::GetUserBalance { user } => to_binary(&query_user_balance(deps, user)?),
    }
}

fn query_project_status(deps: Deps) -> StdResult<ProjectStatusResponse> {
    let state = PROJECT.load(deps.storage)?;
    Ok(ProjectStatusResponse { project_status: state })
}

fn query_user_balance(deps: Deps, user: Option<String>) -> StdResult<UserBalanceResponse> {
    let user_addr = maybe_addr(deps.api, user)?;
    match user_addr {
        Some(addr) => {
            let balance = BACKINGS.may_load(deps.storage, &addr)?;
            match balance {
                None => Ok(UserBalanceResponse { user_balance: Uint128::zero() }),
                Some(balance) => Ok(UserBalanceResponse { user_balance: balance }),
            } 
        }
        None => {
            return Err(StdError::GenericErr {
                msg: "Invalid user address".to_string()
            } );
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Timestamp, Addr};
    use riskless::project::{Project};

    const TOKEN_MULTIPLIER : u128 = 1_000_000;

    #[test]
    fn compound_interest_test() {
        let yield_ = compute_yield(1000. , 365.);
        assert_eq!(yield_, Uint128::from(221u128));
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let sender = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v";
        let project = Project {
            name: "Film 1".to_string(),
            creator: Addr::unchecked(sender),
            creation_date: Timestamp::from_seconds(1662105262),
            project_status: ProjectStatus::FundingInProgress,
            target_principal_amount: Uint128::from(1_000_000_000u128),
            target_yield_amount: Uint128::from(500_000_000u128),
            principal_amount: Uint128::zero(), 
            fund_deadline: Timestamp::from_seconds(1662105262),
        };

        // Initialize new project contract
        let msg = InstantiateMsg { project: project };
        let info = mock_info(sender, &coins(0, "uusd"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query status of project
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProjectStatus { }).unwrap();
        let value: ProjectStatusResponse = from_binary(&res).unwrap();
        assert_eq!(Timestamp::from_seconds(1662105262), value.project_status.fund_deadline);

        // Fund project with 20 uusd
        let fund_project_uusd = Uint128::from(20 * TOKEN_MULTIPLIER);
        let msg = ExecuteMsg::FundProject { };
        let info = mock_info(sender, &coins(fund_project_uusd.u128(), "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query user balance
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUserBalance { user: Some(sender.to_string())}, ).unwrap();
        let value: UserBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(fund_project_uusd, value.user_balance);
    }
}
