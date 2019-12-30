use cosmwasm::mock::mock_params;
use cosmwasm::serde::from_slice;
use cosmwasm::traits::{Api, ReadonlyStorage};
use cosmwasm::types::{coin, ContractResult, CosmosMsg, QueryResult};

use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use {{crate_name}}::contract::{CONFIG_KEY, CountResponse HandleMsg, InitMsg, QueryMsg, State};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.

You can easily convert unit tests to integration tests.
1. First copy them over verbatum,
2. Then change
    let mut deps = dependencies(20);
To
    let mut deps = mock_instance(WASM);
3. If you access raw storage, where ever you see something like:
    deps.storage.get(CONFIG_KEY).expect("no data stored");
 replace it with:
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        //...
    });
4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
5. When matching on error codes, you can not use Error types, but rather must use strings:
     match res {
       Err(Error::Unauthorized{..}) => {},
       _ => panic!("Must return unauthorized error"),
     }
   becomes:
     match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected error"),
     }



**/

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/{{crate_name}}.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");


#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM);

let msg = InitMsg { count: 17 };
let params = mock_params(&deps.api, "creator", &coin("1000", "earth"), &[]);

// we can just call .unwrap() to assert this was a success
let res = init(&mut deps, params, msg).unwrap();
assert_eq!(0, res.messages.len());

// it worked, let's query the state
let mut res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
let value: CountResponse = from_slice(&res).unwrap();
assert_eq!(17, value.count);
}

#[test]
fn increment() {
    let mut deps = mock_instance(WASM);

let msg = InitMsg { count: 17 };
let params = mock_params(&deps.api, "creator", &coin("2", "token"), &coin("2", "token"));
let _res = init(&mut deps, params, msg).unwrap();

// beneficiary can release it
let params = mock_params(&deps.api, "anyone", &coin("2", "token"), &[]);
let msg = HandleMsg::Increment {};
let _res = handle(&mut deps, params, msg).unwrap();

// should increase counter by 1
let mut res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
let value: CountResponse = from_slice(&res).unwrap();
assert_eq!(18, value.count);
}

#[test]
fn reset() {
    let mut deps = mock_instance(WASM);

let msg = InitMsg { count: 17 };
let params = mock_params(&deps.api, "creator", &coin("2", "token"), &coin("2", "token"));
let _res = init(&mut deps, params, msg).unwrap();

// beneficiary can release it
let unauth_params = mock_params(&deps.api, "anyone", &coin("2", "token"), &[]);
let msg = HandleMsg::Reset { count: 5 };
let res = handle(&mut deps, unauth_params, msg);
match res {
ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
_ => panic!("Expected error"),
}

// only the original creator can reset the counter
let auth_params = mock_params(&deps.api, "creator", &coin("2", "token"), &[]);
let msg = HandleMsg::Reset {count: 5};
let _res = handle(&mut deps, auth_params, msg).unwrap();

// should now be 5
let mut res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
let value: CountResponse = from_slice(&res).unwrap();
assert_eq!(5, value.count);
}
