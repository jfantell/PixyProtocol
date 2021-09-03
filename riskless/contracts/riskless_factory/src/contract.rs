#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, DepsMut, Deps, Env, MessageInfo, Addr, Binary,
    Response, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, ReplyOn, Reply, StdError };

use riskless::project::InstantiateMsg as ProjectInstantiateMsg;
use riskless::project::{Project, ProjectStatus};
use riskless::factory::{ExecuteMsg, InstantiateMsg, QueryMsg, ProjectContractAddressResponse};
use crate::error::ContractError;
use crate::state::{ADMIN, PROJECTS, CONFIG, Config, TMP_PROJECT_NAME};
use protobuf::Message;
use crate::response::MsgInstantiateContractResponse;

use cw0::{maybe_addr};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Set admin
    ADMIN.set(deps.branch(), Some(info.sender))?;

    // Set configuration
    let anchor_money_market_address: String = match msg.anchor_money_market_address {
        Some(x) => {x},
        None => { return Err(ContractError::InvalidAddress {} ); }
    };
    let a_ust_address: String = match msg.a_ust_address {
        Some(x) => {x},
        None => { return Err(ContractError::InvalidAddress {} ); }
    };

    let cfg = Config {
        project_code_id : msg.project_code_id,
        anchor_money_market_address : anchor_money_market_address,
        a_ust_address : a_ust_address,
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
            fund_deadline,
        } => create_project(deps, env, info, name, target_principal_amount, target_yield_amount, fund_deadline),
    }
}

pub fn create_project(deps: DepsMut, env: Env, info: MessageInfo, name: String,
    target_principal_amount: Uint128, target_yield_amount : Uint128, 
    fund_deadline: Timestamp) -> Result<Response, ContractError> {
    // Create new project
    let project = Project {
        name: name.clone(),
        creator: info.sender,
        creation_date : env.block.time,
        project_status: ProjectStatus::FundingInProgress,
        target_principal_amount: target_principal_amount,
        target_yield_amount: target_yield_amount,
        principal_amount: Uint128::zero(),
        fund_deadline: fund_deadline
    };

    if let Ok(Some(_)) = PROJECTS.may_load(deps.storage, &name.as_bytes()) {
        return Err(ContractError::UnableToCreateNewProject {} );
    }

    let config: Config = CONFIG.load(deps.storage)?;

    let contract_address = env.contract.address.to_string();

    TMP_PROJECT_NAME.save(deps.storage, &name)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_project"),
            ("name", &format!("{}", name)),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.project_code_id,
                funds: vec![],
                admin: Some(contract_address),
                label: "".to_string(),
                msg: to_binary(&ProjectInstantiateMsg {
                    project: project
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError>  {
    match msg.id {
        1 => { create_new_project(deps, _env, msg) }
        _ => { return Err(ContractError::UnableToCreateNewProject {} ); }
    }
}

pub fn create_new_project(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError>  {
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(msg.result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let project_contract_address = Addr::unchecked(String::from(res.get_contract_address()));
    let project_name = TMP_PROJECT_NAME.load(deps.storage)?;
    
    PROJECTS.update(deps.storage, project_name.as_bytes(), | _address | -> Result<_, ContractError> {
        Ok(project_contract_address)
    })?;

    Ok(Response::new()
    .add_attributes(vec![
        ("action", "create_project"),
        ("name", &format!("{}", project_name)),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProjectContractAddress { name } => to_binary(&query_project_status(deps, name )?),
    }
}

fn query_project_status(deps: Deps, name : String ) -> StdResult<ProjectContractAddressResponse> {
    let address = PROJECTS.load(deps.storage, name.as_bytes())?;
    Ok(ProjectContractAddressResponse { address: address })
}