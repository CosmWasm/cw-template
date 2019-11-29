use std::str::from_utf8;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{Params, QueryResponse, RawQuery, Response};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: String,
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
    Raw(RawQuery),
}

// raw_query is a helper to generate a serialized format of a raw_query
// meant for test code and integration tests
pub fn raw_query(key: &[u8]) -> Result<Vec<u8>> {
    let key = from_utf8(key).context(Utf8Err {})?.to_string();
    to_vec(&QueryMsg::Raw(RawQuery { key })).context(SerializeErr { kind: "QueryMsg" })
}

pub static CONFIG_KEY: &[u8] = b"config";

pub fn init<T: Storage>(store: &mut T, params: Params, msg: Vec<u8>) -> Result<Response> {
    let msg: InitMsg = from_slice(&msg).context(ParseErr { kind: "InitMsg" })?;
    store.set(
        CONFIG_KEY,
        &to_vec(&State {
            count: msg.count,
            owner: params.message.signer,
        })
            .context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn handle<T: Storage>(store: &mut T, params: Params, msg: Vec<u8>) -> Result<Response> {
    let msg: HandleMsg = from_slice(&msg).context(ParseErr { kind: "HandleMsg" })?;
    let data = store.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;

    match msg {
        HandleMsg::Increment {} => try_increment(store, params, state),
        HandleMsg::Reset { count } => try_reset(store, params, state, count),
    }
}

pub fn try_increment<T: Storage>(
    store: &mut T,
    _params: Params,
    mut state: State,
) -> Result<Response> {
    state.count += 1;
    store.set(
        CONFIG_KEY,
        &to_vec(&state).context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn try_reset<T: Storage>(
    store: &mut T,
    params: Params,
    mut state: State,
    count: i32,
) -> Result<Response> {
    if params.message.signer != state.owner {
        Unauthorized {}.fail()
    } else {
        state.count = count;
        store.set(
            CONFIG_KEY,
            &to_vec(&state).context(SerializeErr { kind: "State" })?,
        );
        Ok(Response::default())
    }
}

pub fn query<T: Storage>(store: &T, msg: Vec<u8>) -> Result<QueryResponse> {
    let msg: QueryMsg = from_slice(&msg).context(ParseErr { kind: "QueryMsg" })?;
    match msg {
        QueryMsg::Raw(raw) => perform_raw_query(store, raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::errors::Error;
    use cosmwasm::mock::MockStorage;
    use cosmwasm::types::{coin, mock_params};

    #[test]
    fn proper_initialization() {
        let mut store = MockStorage::new();
        let msg = to_vec(&InitMsg { count: 17 }).unwrap();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        // we can just call .unwrap() to assert this was a success
        let res = init(&mut store, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let mut q_res = query(&store, raw_query(CONFIG_KEY).unwrap()).unwrap();
        let model = q_res.results.pop().expect("no data stored");
        let state: State = from_slice(&model.val).unwrap();
        assert_eq!(
            state,
            State {
                owner: String::from("creator"),
                count: 17,
            }
        );
    }

    #[test]
    fn fails_on_bad_init() {
        let mut store = MockStorage::new();
        let bad_msg = b"{}".to_vec();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let res = init(&mut store, params, bad_msg);
        assert_eq!(true, res.is_err());
    }

    #[test]
    fn increment() {
        let mut store = MockStorage::new();
        let msg = to_vec(&InitMsg { count: 17 }).unwrap();
        let params = mock_params("creator", &coin("2", "token"), &coin("2", "token"));
        let _res = init(&mut store, params, msg).unwrap();

        // beneficiary can release it
        let params = mock_params("anyone", &coin("2", "token"), &[]);
        let msg = r#"{"increment":{}}"#.as_bytes();
        let _res = handle(&mut store, params, msg.to_vec()).unwrap();

        // should increase counter by 1
        let mut q_res = query(&store, raw_query(CONFIG_KEY).unwrap()).unwrap();
        let model = q_res.results.pop().expect("no data stored");
        let state: State = from_slice(&model.val).unwrap();
        assert_eq!(
            state,
            State {
                owner: String::from("creator"),
                count: 18,
            }
        );
    }

    #[test]
    fn reset() {
        let mut store = MockStorage::new();
        let msg = to_vec(&InitMsg { count: 17 }).unwrap();
        let params = mock_params("creator", &coin("2", "token"), &coin("2", "token"));
        let _res = init(&mut store, params, msg).unwrap();

        // beneficiary can release it
        let unauth_params = mock_params("anyone", &coin("2", "token"), &[]);
        let msg = r#"{"reset":{"count": 5}}"#.as_bytes();
        let res = handle(&mut store, unauth_params, msg.to_vec());
        match res {
            Err(Error::Unauthorized{..}) => {},
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_params = mock_params("creator", &coin("2", "token"), &[]);
        let msg = to_vec(&HandleMsg::Reset {count: 5}).unwrap();
        let _res = handle(&mut store, auth_params, msg).unwrap();

        // should increase counter by 1
        let mut q_res = query(&store, raw_query(CONFIG_KEY).unwrap()).unwrap();
        let model = q_res.results.pop().expect("no data stored");
        let state: State = from_slice(&model.val).unwrap();
        assert_eq!(
            state,
            State {
                owner: String::from("creator"),
                count: 5,
            }
        );
    }
}
