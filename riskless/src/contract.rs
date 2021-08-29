#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Timestamp, Uint128, StdError };

use crate::error::ContractError;
use crate::msg::{UserBalanceResponse, ProjectStatusResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ProjectStatus, Project, PROJECTS, BALANCES};
use crate::state::{ADMIN, CONFIG, Config};
use crate::anchor::{deposit_stable_msg};

use cw0::{maybe_addr};

const BASE_AMOUNT : u128 = 1_000_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    ADMIN.set(deps.branch(), maybe_addr(api, msg.admin)?)?;

    let anchor_money_market_address: String = match msg.anchor_money_market_address {
        Some(x) => {x},
        None => { return Err(ContractError::InvalidAddress {} ); }
    };
    let a_ust_address: String = match msg.a_ust_address {
        Some(x) => {x},
        None => { return Err(ContractError::InvalidAddress {} ); }
    };

    let cfg = Config {
        anchor_money_market_address : anchor_money_market_address,
        a_ust_address : a_ust_address
    };
    CONFIG.save(deps.storage, &cfg)?;
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
        ExecuteMsg::CreateProject {
            name,
            target_principal_amount,
            target_yield_amount,
            project_deadline,
        } => create_project(deps, env, info, name, target_principal_amount, target_yield_amount, project_deadline),
        ExecuteMsg::FundProject {
            name,
        } => fund_project(deps, info, name),
        ExecuteMsg::WidthdrawPrincipal {
            name,
        } => withdraw_principal(deps, env, info, name),
        ExecuteMsg::ChangeProjectStatus {
            name,
            project_status
        } => change_status(deps, env, info, name, project_status),
        ExecuteMsg::WidthdrawYield {
            name,
        } => withdraw_yield(deps, info, name) 
    }
}

pub fn create_project(deps: DepsMut, env: Env, info: MessageInfo, name: String,
    target_principal_amount: Uint128, target_yield_amount : Uint128, 
    project_deadline: Timestamp) -> Result<Response, ContractError> {
    // Create new project
    let project = Project {
        creator: info.sender,
        creation_date : env.block.time,
        project_status: ProjectStatus::FundingInProgress,
        target_principal_amount: target_principal_amount,
        target_yield_amount: target_yield_amount,
        principal_amount: Uint128::zero(),
        project_deadline: project_deadline
    };

    // Save new project
    PROJECTS.save(deps.storage, name.as_bytes(), &project)?;

    Ok(Response::new()
        .add_attribute("method", "create_project")
        .add_attribute("project_name", name)
        .add_attribute("target_principal_amount", target_principal_amount.to_string())
        .add_attribute("target_yield_amount", target_yield_amount.to_string()))
}
pub fn fund_project(deps: DepsMut, info: MessageInfo, name: String) -> Result<Response, ContractError> {
    let state =  PROJECTS.load(deps.storage, name.as_bytes())?;
    // If project off track, prevent backers from funding
    if state.project_status == ProjectStatus::ProjectOffTrack {
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
    BALANCES.update(deps.storage, (&info.sender, name.as_bytes()), | current_amount | -> Result<_, ContractError> {
        Ok(current_amount.unwrap_or_default() + deposit_amount)
    })?;

    // Update project stats
    PROJECTS.update(deps.storage, name.as_bytes(), | project | -> Result<_, ContractError> {
        match project {
            Some(mut p) => {
                p.principal_amount += deposit_amount;
                Ok(p)
            },
            None => {
                return Err(ContractError::UnableToUpdateContractState {} )
            }
        }
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "fund_project")
        .add_attribute("project_name", name)
        .add_attribute("amount_uusd", deposit_amount.to_string())
        .add_attribute("sender", info.sender))
}

pub fn withdraw_principal(deps: DepsMut, env: Env, info: MessageInfo, name: String) -> Result<Response, ContractError> {
    let state = PROJECTS.load(deps.storage, name.as_bytes())?;
    let withdraw_amount = BALANCES.load(deps.storage, (&info.sender, name.as_bytes()))?;

    // Backers cannot withdraw principal if the following is true:
    // project status is TargetMet && yield target has not been met
    let yield_ = match get_yield_amount(&deps, &env, &info, &name) {
        Ok(y) => { y },
        Err(_) => { return Err(ContractError::UnableToAcquireYield {} ) }
    };
    
    if (state.project_status == ProjectStatus::TargetMet) && ( yield_ < state.target_yield_amount ) {
        return Ok(Response::new()
            .add_attribute("action", "widthdraw_principal")
            .add_attribute("status", "cannot withdraw principal: target funding met and yield less than target yield")
            .add_attribute("sender", info.sender));
    }
    
    BALANCES.update(deps.storage, (&info.sender, name.as_bytes()), | current_amount | -> Result<_, ContractError> {
        Ok(current_amount.unwrap_or_default() - withdraw_amount)
    })?;
    
    PROJECTS.update(deps.storage, name.as_bytes(), | project | -> Result<_, ContractError> {
        match project {
            Some(mut p) => {
                p.principal_amount -= withdraw_amount;
                Ok(p)
            },
            None => {
                return Err(ContractError::UnableToUpdateContractState {} )
            }
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "widthdraw_principal")
        .add_attribute("project_name", name)
        .add_attribute("amount_uusd", withdraw_amount.to_string())
        .add_attribute("sender", info.sender))
}

pub fn get_yield_amount(deps: &DepsMut, env: &Env, info: &MessageInfo, name: &String) -> Result<Uint128, ContractError> {
    let state = PROJECTS.load(deps.storage, name.as_bytes())?;
    let start_amount = state.principal_amount;
    let creation_date = state.creation_date;
    let current_date = env.block.time;
    let daily_rate = 0.05479;
    let number_of_days = current_date.minus_seconds(creation_date.seconds()).seconds() / (60 * 60 * 24);
    let yield_ = (start_amount * (1.0+daily_rate)^number_of_days - start_amount);
    return yield_;
}

pub fn change_status(deps: DepsMut, env: Env, info: MessageInfo, name: String, project_status: Option<ProjectStatus>) -> Result<Response, ContractError> {
    let mut state = PROJECTS.load(deps.storage, name.as_bytes())?;

    // Admin can change project to ProjectOffTrack at any time
    if ADMIN.is_admin(deps.as_ref(), &info.sender)? {
        match project_status.clone() {
            Some(status) => { state.project_status = status }
            None => {}
        }
    }

    // Project deadline and principal amount target has been met, change status to TargetMet
    // users can no longer widthdraw principal
    if env.block.time >= state.project_deadline && state.project_status == ProjectStatus::FundingInProgress {
        if state.principal_amount >= state.target_principal_amount {
            state.project_status = ProjectStatus::TargetMet;
        }
        else {
            state.project_status = ProjectStatus::ProjectOffTrack;
        }
    }

    PROJECTS.save(deps.storage, name.as_bytes(), &state)?;
    
    return Ok(Response::new()
        .add_attribute("action", "change_status")
        .add_attribute("project_name", name)
        .add_attribute("status", format!("{:?}", project_status.clone()))
        .add_attribute("sender", info.sender));
}

pub fn withdraw_yield(deps: DepsMut, info: MessageInfo, name: String) -> Result<Response, ContractError> {

    let state = PROJECTS.load(deps.storage, name.as_bytes())?;
    if (state.creator == info.sender) || ADMIN.is_admin(deps.as_ref(), &info.sender)? {}

    Ok(Response::new()
        .add_attribute("action", "widthdraw_yield")
        .add_attribute("project_name", name)
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectStatus { name } => to_binary(&query_project_status(deps, name )?),
        QueryMsg::GetUserBalance { name, user } => to_binary(&query_user_balance(deps, name, user)?),
    }
}

fn query_project_status(deps: Deps, name : String ) -> StdResult<ProjectStatusResponse> {
    let state = PROJECTS.load(deps.storage, name.as_bytes())?;
    Ok(ProjectStatusResponse { project_status: state })
}

fn query_user_balance(deps: Deps, name : String, user: Option<String>) -> StdResult<UserBalanceResponse> {
    let user_addr = maybe_addr(deps.api, user)?;
    match user_addr {
        Some(addr) => {
            let balance = BALANCES.load(deps.storage, (&addr, name.as_bytes()))?;
            Ok(UserBalanceResponse { user_balance: balance })
        }
        None => {
            return Err(StdError::GenericErr {
                msg: "Invalid user address".to_string()
            } );
        }
    }
    
}

// fn query_project_yield(deps: Deps, name : String) -> StdResult<UserBalanceResponse> {
//     //

//     // match user_addr {
//     //     Some(addr) => {
//     //         let balance = BALANCES.load(deps.storage, (&addr, name.as_bytes()))?;
//     //         Ok(UserBalanceResponse { user_balance: balance })
//     //     }
//     //     None => {
//     //         return Err(StdError::GenericErr {
//     //             msg: "Invalid user address".to_string()
//     //         } );
//     //     }
//     // }
    
// }

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
        let msg = InstantiateMsg { admin: Some(sender.to_string()), anchor_money_market_address: Some(sender.to_string()), a_ust_address: Some(sender.to_string()) };
        let info = mock_info(sender, &coins(0, "uusd"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        let target_principal_amount = Uint128::from(1_000_000 * TOKEN_MULTIPLIER);
        // 20% of target_principal_amount
        let target_yield_amount = target_principal_amount.multiply_ratio(20u128, 1u128)
            .checked_div(Uint128::from(100u128));
        let project_deadline = Timestamp::from_seconds(1633028107);

        // Create new project with id: film1
        let project_name = "film1".to_string();
        let msg = ExecuteMsg::CreateProject {
            name: project_name.clone(),
            target_principal_amount: target_principal_amount,
            target_yield_amount : target_yield_amount.unwrap(),
            project_deadline : project_deadline
        };
        let info = mock_info(sender, &coins(0, "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query status of recently created project with id: film1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProjectStatus { name : project_name.clone() }).unwrap();
        let value: ProjectStatusResponse = from_binary(&res).unwrap();
        assert_eq!(project_deadline, value.project_status.project_deadline);

        // Fund project with 1000 uusd
        let fund_project_uusd = Uint128::from(1_000 * TOKEN_MULTIPLIER);
        let msg = ExecuteMsg::FundProject {
            name: project_name.clone(),
        };
        let info = mock_info(sender, &coins(fund_project_uusd.u128(), "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        print!("{:?}\n", res);

        // Query user balance
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetUserBalance { name : project_name.clone(), user: Some(sender.to_string())}, ).unwrap();
        let value: UserBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(fund_project_uusd, value.user_balance);
    }
}
