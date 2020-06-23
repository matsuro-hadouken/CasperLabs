use engine_test_support::{
    internal::{ExecuteRequestBuilder, WasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST},
    DEFAULT_ACCOUNT_ADDR,
};
use types::{account::AccountHash, runtime_args, RuntimeArgs, U512};

const CONTRACT_MINT_PURSE: &str = "mint_purse.wasm";
const CONTRACT_TRANSFER_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";
const SYSTEM_ADDR: AccountHash = AccountHash::new([0u8; 32]);
const TRANSFER_AMOUNT: u64 = 250_000_000 + 1000;

#[ignore]
#[test]
fn should_run_mint_purse_contract() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { "target" =>SYSTEM_ADDR, "amount" => U512::from(TRANSFER_AMOUNT) },
    )
    .build();
    let exec_request_2 =
        ExecuteRequestBuilder::standard(SYSTEM_ADDR, CONTRACT_MINT_PURSE, RuntimeArgs::default())
            .build();

    let mut builder = WasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder.exec(exec_request_1).commit().expect_success();
    builder.exec(exec_request_2).commit().expect_success();
}

#[ignore]
#[test]
fn should_not_allow_non_system_accounts_to_mint() {
    let exec_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_MINT_PURSE,
        RuntimeArgs::default(),
    )
    .build();

    assert!(WasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .commit()
        .is_error());
}
