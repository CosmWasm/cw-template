#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{% raw %}{{% endraw %}{% unless minimal %}to_binary, {% endunless %}Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
{% if minimal %}// {% endif %}use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, {% unless minimal %}GetCountResponse, {% endunless %}InstantiateMsg, QueryMsg};
{% unless minimal %}use crate::state::{State, STATE};
{% endunless %}
{% if minimal %}/*
{% endif %}// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
{% if minimal %}*/
{% endif %}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    {% if minimal %}_{% endif %}deps: DepsMut,
    _env: Env,
    {% if minimal %}_{% endif %}info: MessageInfo,
    {% if minimal %}_{% endif %}msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    {% if minimal %}unimplemented!(){% else %}let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string())){% endif %}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    {% if minimal %}_{% endif %}deps: DepsMut,
    _env: Env,
    {% if minimal %}_{% endif %}info: MessageInfo,
    {% if minimal %}_{% endif %}msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    {% if minimal %}unimplemented!(){% else %}match msg {
        ExecuteMsg::Increment {} => execute::increment(deps),
        ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
    }{% endif %}
}{% unless minimal %}

pub mod execute {
    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }
}{% endunless %}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query({% if minimal %}_{% endif %}deps: Deps, _env: Env, {% if minimal %}_{% endif %}msg: QueryMsg) -> StdResult<Binary> {
    {% if minimal %}unimplemented!(){% else %}match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
    }{% endif %}
}{% unless minimal %}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}{% endunless %}

#[cfg(test)]
mod tests {% raw %}{{% endraw %}{% unless minimal %}
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
{% endunless %}}
