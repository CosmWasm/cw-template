use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

{% if minimal %}use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};{% else %}use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};{% endif %}

{% if minimal %}use crate::msg::ExecuteMsg;{% else %}use crate::msg::{ExecuteMsg, GetCountResponse, QueryMsg};{% endif %}

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }{% unless minimal %}

    /// Get Count
    pub fn count<Q, T, CQ>(&self, querier: &Q) -> StdResult<GetCountResponse>
    where
        Q: Querier,
        T: Into<String>,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetCount {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();
        let res: GetCountResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }{% endunless %}
}
