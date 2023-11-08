use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::Empty;
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct ContractInterface;

impl<Chain: CwEnv> Uploadable for ContractInterface<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("{{crate_name}}")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{ExecuteMsgFns, QueryMsgFns};
    use cw_orch::anyhow;

    #[test]
    fn contract_logic() -> anyhow::Result<()> {
        let mock = Mock::new(&Addr::unchecked("sender"));
        let contract = ContractInterface::new("project-name", mock);
        contract.upload()?;

        contract.instantiate(&InstantiateMsg { count: 7 }, None, None)?;
        assert_eq!(contract.get_count()?.count, 7);

        contract.increment()?;
        assert_eq!(contract.get_count()?.count, 8);

        Ok(())
    }
}
