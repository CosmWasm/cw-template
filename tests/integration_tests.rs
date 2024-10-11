use {{project-name | snake_case}}::{
    msg::{GetCountResponse, InstantiateMsg, QueryMsg},
    ContractError, interface::ContractInterface,
};
// Use prelude to get all the necessary imports
use cw_orch::prelude::*;
use cw_orch::anyhow;

use cosmwasm_std::Addr;

// consts for testing
const USER: &str = "user";
const ADMIN: &str = "admin";

#[test]
fn count() -> anyhow::Result<()> {
    // Create a user
    let user = Addr::unchecked(USER);
    // Create the mock. This will be our chain object throughout
    let mock = Mock::new(ADMIN);

    // Set up the contract (Definition below) ↓↓
    let contract = setup(mock.clone())?;

    // Increment the count of the contract
    contract
        // Set the caller to user
        .call_as(&user)
        // Call the increment function (auto-generated function provided by ExecuteMsgFns)
        .increment()?;

    // Get the count.
    use {{project-name | snake_case}}::msg::QueryMsgFns;
    let count1 = contract.get_count()?;

    // or query it manually
    let count2: GetCountResponse = contract.query(&QueryMsg::GetCount {})?;
    assert_eq!(count1.count, count2.count);

    // Or get it manually from the chain
    let count3: GetCountResponse = mock.query(&QueryMsg::GetCount {}, &contract.address()?)?;
    assert_eq!(count1.count, count3.count);

    // Check the count
    assert_eq!(count1.count, 2);
    // Reset
    use {{project-name | snake_case}}::msg::ExecuteMsgFns;
    contract.reset(0)?;

    let count = contract.get_count()?;
    assert_eq!(count.count, 0);

    // Check negative case
    let exec_res: Result<cw_orch::mock::cw_multi_test::AppResponse, CwOrchError> =
        contract.call_as(&user).reset(0);

    let expected_err = ContractError::Unauthorized {};
    assert_eq!(
        exec_res.unwrap_err().downcast::<ContractError>()?,
        expected_err
    );

    Ok(())
}

/// Instantiate the contract in any CosmWasm environment
fn setup<Chain: CwEnv>(chain: Chain) -> anyhow::Result<ContractInterface<Chain>> {
    // Construct the  interface
    let contract = ContractInterface::new(chain.clone());
    let admin = Addr::unchecked(ADMIN);

    // Upload the contract
    let upload_resp = contract.upload()?;

    // Get the code-id from the response.
    let code_id = upload_resp.uploaded_code_id()?;
    // or get it from the interface.
    assert_eq!(code_id, contract.code_id()?);

    // Instantiate the contract
    let msg = InstantiateMsg { count: 1i32 };
    let init_resp = contract.instantiate(&msg, Some(&admin), &[])?;

    // Get the address from the response
    let contract_addr = init_resp.instantiated_contract_address()?;
    // or get it from the interface.
    assert_eq!(contract_addr, contract.address()?);

    // Return the interface
    Ok(contract)
}
