use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized};
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{CanonicalAddr, Params, Response};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HandleMsg {
    Increment {},
    Reset { count: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

pub static CONFIG_KEY: &[u8] = b"config";

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: InitMsg,
) -> Result<Response> {
    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&State {
            count: msg.count,
            owner: params.message.signer,
        })
        .context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: HandleMsg,
) -> Result<Response> {
    let data = deps.storage.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;

    match msg {
        HandleMsg::Increment {} => try_increment(deps, params, state),
        HandleMsg::Reset { count } => try_reset(deps, params, state, count),
    }
}

pub fn try_increment<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    _params: Params,
    mut state: State,
) -> Result<Response> {
    state.count += 1;
    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&state).context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn try_reset<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    mut state: State,
    count: i32,
) -> Result<Response> {
    if params.message.signer != state.owner {
        Unauthorized {}.fail()
    } else {
        state.count = count;
        deps.storage.set(
            CONFIG_KEY,
            &to_vec(&state).context(SerializeErr { kind: "State" })?,
        );
        Ok(Response::default())
    }
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::GetCount {} => query_count(deps),
    }
}

fn query_count<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let data = deps.storage.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;
    let resp = CountResponse { count: state.count };
    to_vec(&resp).context(SerializeErr {
        kind: "CountResponse",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::errors::Error;
    use cosmwasm::mock::{dependencies, mock_params};
    use cosmwasm::types::coin;

    #[test]
    fn proper_initialization() {
        let mut deps = dependencies(20);

        let msg = InitMsg { count: 17 };
        let params = mock_params(&deps.api, "creator", &coin("1000", "earth"), &[]);

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_slice(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = dependencies(20);

        let msg = InitMsg { count: 17 };
        let params = mock_params(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, params, msg).unwrap();

        // beneficiary can release it
        let params = mock_params(&deps.api, "anyone", &coin("2", "token"), &[]);
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, params, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_slice(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = dependencies(20);

        let msg = InitMsg { count: 17 };
        let params = mock_params(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, params, msg).unwrap();

        // beneficiary can release it
        let unauth_params = mock_params(&deps.api, "anyone", &coin("2", "token"), &[]);
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_params, msg);
        match res {
            Err(Error::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_params = mock_params(&deps.api, "creator", &coin("2", "token"), &[]);
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_params, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_slice(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
