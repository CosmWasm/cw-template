use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::Empty;
use cw_orch::interface;
use cw_orch::prelude::*;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id = "{{project-name}}")]
pub struct {{project-name | upper_camel_case}}I;

impl<Chain: CwEnv> Uploadable for {{project-name | upper_camel_case}}I<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_info: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("{{crate_name}}")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }
}

impl<Chain: CwEnv> {{project-name | upper_camel_case}}I<Chain> {
    /// Instantiate the contract in any CosmWasm environment
    pub fn setup(chain: Chain, admin: Addr) -> cw_orch::anyhow::Result<{{project-name | upper_camel_case}}I<Chain>> {
        // Construct the interface
        let contract = {{project-name | upper_camel_case}}I::new(chain.clone());

        // Upload the contract
        contract.upload()?;

        // Instantiate the contract
        let msg = InstantiateMsg {% raw %}{{% endraw %}{% unless minimal %} count: 1i32 {% endunless %}};
        contract.instantiate(&msg, Some(&admin), &[])?;

        // Return the interface
        Ok(contract)
    }
}
