use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::types::{coin, mock_params, ContractResult};
use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use {{crate_name}}::contract::{CONFIG_KEY, HandleMsg, InitMsg, State, raw_query};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with: `cargo wasm`
Then running `cargo test` will validate we can properly call into that generated data.

You can copy the code from unit tests here verbatim, then make a few changes:

Replace `let mut store = MockStorage::new();` with `let mut store = mock_instance(WASM);`.

Replace `query(&store...` with `query(&mut store..` (we need mutability to pass args into wasm).

Any switches on error results, using types will have to use raw strings from formatted errors.
You can use a pattern like this to assert specific errors:

```
match res {
    ContractResult::Err(msg) => assert_eq!(msg, "Contract error: creating expired escrow"),
    _=> panic!("expected error"),
}
```
**/

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/{{crate_name}}.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");


#[test]
fn proper_initialization() {
    let mut store = mock_instance(WASM);
    let msg = to_vec(&InitMsg { count: 17 }).unwrap();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    // we can just call .unwrap() to assert this was a success
    let res = init(&mut store, params, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let mut q_res = query(&mut store, raw_query(CONFIG_KEY).unwrap()).unwrap();
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
    let mut store = mock_instance(WASM);
    let bad_msg = b"{}".to_vec();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let res = init(&mut store, params, bad_msg);
    assert_eq!(true, res.is_err());
}

#[test]
fn increment() {
    let mut store = mock_instance(WASM);
    let msg = to_vec(&InitMsg { count: 17 }).unwrap();
    let params = mock_params("creator", &coin("2", "token"), &coin("2", "token"));
    let _res = init(&mut store, params, msg).unwrap();

    // beneficiary can release it
    let params = mock_params("anyone", &coin("2", "token"), &[]);
    let msg = r#"{"increment":{}}"#.as_bytes();
    let _res = handle(&mut store, params, msg.to_vec()).unwrap();

    // should increase counter by 1
    let mut q_res = query(&mut store, raw_query(CONFIG_KEY).unwrap()).unwrap();
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
    let mut store = mock_instance(WASM);
    let msg = to_vec(&InitMsg { count: 17 }).unwrap();
    let params = mock_params("creator", &coin("2", "token"), &coin("2", "token"));
    let _res = init(&mut store, params, msg).unwrap();

    // beneficiary can release it
    let unauth_params = mock_params("anyone", &coin("2", "token"), &[]);
    let msg = r#"{"reset":{"count": 5}}"#.as_bytes();
    let res = handle(&mut store, unauth_params, msg.to_vec());
    match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Must return unauthorized error"),
    }

    // only the original creator can reset the counter
    let auth_params = mock_params("creator", &coin("2", "token"), &[]);
    let msg = to_vec(&HandleMsg::Reset {count: 5}).unwrap();
    let _res = handle(&mut store, auth_params, msg).unwrap();

    // should increase counter by 1
    let mut q_res = query(&mut store, raw_query(CONFIG_KEY).unwrap()).unwrap();
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
