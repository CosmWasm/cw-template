use snafu::ResultExt;

use cosmwasm::errors::{Result, SerializeErr, Unauthorized};
use cosmwasm::serde::to_vec;
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{Params, Response};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: InitMsg,
) -> Result<Response> {
    let state = State {
        count: msg.count,
        owner: params.message.signer,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: HandleMsg,
) -> Result<Response> {
    match msg {
        HandleMsg::Increment {} => try_increment(deps, params),
        HandleMsg::Reset { count } => try_reset(deps, params, count),
    }
}

pub fn try_increment<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    _params: Params,
) -> Result<Response> {
    config(&mut deps.storage).update(&|mut state| {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_reset<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    count: i32,
) -> Result<Response> {
    config(&mut deps.storage).update(&|mut state| {
        if params.message.signer != state.owner {
            Unauthorized {}.fail()?;
        }

        state.count = count;
        Ok(state)
    })?;
    Ok(Response::default())
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::GetCount {} => query_count(deps),
    }
}

fn query_count<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let state = config_read(&deps.storage).load()?;

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
    use cosmwasm::serde::from_slice;
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
