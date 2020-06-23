use lazy_static::lazy_static;

use engine_core::{
    engine_state::{Error as CoreError, CONV_RATE},
    execution::Error as ExecError,
};
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{
        utils, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{
    account::AccountHash, runtime_args, AccessRights, ApiError, Key, RuntimeArgs, URef, U512,
};

const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const CONTRACT_TRANSFER_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";
const TRANSFER_RESULT_NAMED_KEY: &str = "transfer_result";

lazy_static! {
    static ref TRANSFER_1_AMOUNT: U512 = U512::from(250_000_000) + 1000;
    static ref TRANSFER_2_AMOUNT: U512 = U512::from(750);
    static ref TRANSFER_2_AMOUNT_WITH_ADV: U512 = *DEFAULT_PAYMENT + *TRANSFER_2_AMOUNT;
    static ref TRANSFER_TOO_MUCH: U512 = U512::from(u64::max_value());
    static ref ACCOUNT_1_INITIAL_BALANCE: U512 = *DEFAULT_PAYMENT;
}

const ACCOUNT_1_ADDR: AccountHash = AccountHash::new([1u8; 32]);
const ACCOUNT_2_ADDR: AccountHash = AccountHash::new([2u8; 32]);
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

#[ignore]
#[test]
fn should_transfer_to_account() {
    let initial_genesis_amount: U512 = U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE);
    let transfer_amount: U512 = *TRANSFER_1_AMOUNT;

    // Run genesis
    let mut builder = InMemoryWasmTestBuilder::default();

    let builder = builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get account");

    let default_account_purse = default_account.main_purse();

    // Check genesis account balance
    let genesis_balance = builder.get_purse_balance(default_account_purse);

    assert_eq!(genesis_balance, initial_genesis_amount,);

    // Exec transfer contract

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_1_ADDR, ARG_AMOUNT => *TRANSFER_1_AMOUNT },
    )
    .build();

    builder.exec(exec_request_1).expect_success().commit();

    let account = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account");
    let account_purse = account.main_purse();

    // Check genesis account balance

    let genesis_balance = builder.get_purse_balance(default_account_purse);

    let gas_cost =
        Motes::from_gas(builder.exec_costs(0)[0], CONV_RATE).expect("should convert gas to motes");

    assert_eq!(
        genesis_balance,
        initial_genesis_amount - gas_cost.value() - transfer_amount
    );

    // Check account 1 balance

    let account_1_balance = builder.get_purse_balance(account_purse);

    assert_eq!(account_1_balance, transfer_amount,);
}

#[ignore]
#[test]
fn should_transfer_from_account_to_account() {
    let initial_genesis_amount: U512 = U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE);
    let transfer_1_amount: U512 = *TRANSFER_1_AMOUNT;
    let transfer_2_amount: U512 = *TRANSFER_2_AMOUNT;

    // Run genesis
    let mut builder = InMemoryWasmTestBuilder::default();

    let builder = builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get account");

    let default_account_purse = default_account.main_purse();

    // Check genesis account balance
    let genesis_balance = builder.get_purse_balance(default_account_purse);

    assert_eq!(genesis_balance, initial_genesis_amount,);

    // Exec transfer 1 contract

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_1_ADDR, ARG_AMOUNT => *TRANSFER_1_AMOUNT },
    )
    .build();

    builder.exec(exec_request_1).expect_success().commit();

    let exec_1_response = builder
        .get_exec_response(0)
        .expect("should have exec response");

    let genesis_balance = builder.get_purse_balance(default_account_purse);

    let gas_cost = Motes::from_gas(utils::get_exec_costs(exec_1_response)[0], CONV_RATE)
        .expect("should convert");

    assert_eq!(
        genesis_balance,
        initial_genesis_amount - gas_cost.value() - transfer_1_amount
    );

    // Check account 1 balance
    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("should have account 1");
    let account_1_purse = account_1.main_purse();
    let account_1_balance = builder.get_purse_balance(account_1_purse);

    assert_eq!(account_1_balance, transfer_1_amount,);

    // Exec transfer 2 contract

    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_2_ADDR, ARG_AMOUNT => *TRANSFER_2_AMOUNT },
    )
    .build();

    builder.exec(exec_request_2).expect_success().commit();

    let exec_2_response = builder
        .get_exec_response(1)
        .expect("should have exec response");

    let account_2 = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should have account 2");

    let account_2_purse = account_2.main_purse();

    // Check account 1 balance

    let account_1_balance = builder.get_purse_balance(account_1_purse);

    let gas_cost = Motes::from_gas(utils::get_exec_costs(exec_2_response)[0], CONV_RATE)
        .expect("should convert");

    assert_eq!(
        account_1_balance,
        transfer_1_amount - gas_cost.value() - transfer_2_amount
    );

    let account_2_balance = builder.get_purse_balance(account_2_purse);

    assert_eq!(account_2_balance, transfer_2_amount,);
}

#[ignore]
#[test]
fn should_transfer_to_existing_account() {
    let initial_genesis_amount: U512 = U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE);
    let transfer_1_amount: U512 = *TRANSFER_1_AMOUNT;
    let transfer_2_amount: U512 = *TRANSFER_2_AMOUNT;

    // Run genesis
    let mut builder = InMemoryWasmTestBuilder::default();

    let builder = builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get account");

    let default_account_purse = default_account.main_purse();

    // Check genesis account balance
    let genesis_balance = builder.get_purse_balance(default_account_purse);

    assert_eq!(genesis_balance, initial_genesis_amount,);

    // Exec transfer 1 contract

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_1_ADDR, ARG_AMOUNT => *TRANSFER_1_AMOUNT },
    )
    .build();

    builder.exec(exec_request_1).expect_success().commit();

    // Exec transfer contract

    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account");

    let account_1_purse = account_1.main_purse();

    // Check genesis account balance

    let genesis_balance = builder.get_purse_balance(default_account_purse);

    let gas_cost =
        Motes::from_gas(builder.exec_costs(0)[0], CONV_RATE).expect("should convert gas to motes");

    assert_eq!(
        genesis_balance,
        initial_genesis_amount - gas_cost.value() - transfer_1_amount
    );

    // Check account 1 balance

    let account_1_balance = builder.get_purse_balance(account_1_purse);

    assert_eq!(account_1_balance, transfer_1_amount,);

    // Exec transfer contract

    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_2_ADDR, ARG_AMOUNT => *TRANSFER_2_AMOUNT },
    )
    .build();
    builder.exec(exec_request_2).expect_success().commit();

    let account_2 = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should get account");

    let account_2_purse = account_2.main_purse();

    // Check account 1 balance

    let account_1_balance = builder.get_purse_balance(account_1_purse);

    let gas_cost =
        Motes::from_gas(builder.exec_costs(1)[0], CONV_RATE).expect("should convert gas to motes");

    assert_eq!(
        account_1_balance,
        transfer_1_amount - gas_cost.value() - transfer_2_amount,
    );

    // Check account 2 balance

    let account_2_balance_transform = builder.get_purse_balance(account_2_purse);

    assert_eq!(account_2_balance_transform, transfer_2_amount);
}

#[ignore]
#[test]
fn should_fail_when_insufficient_funds() {
    // Run genesis

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_1_ADDR, ARG_AMOUNT => *TRANSFER_1_AMOUNT },
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_2_ADDR, ARG_AMOUNT => *TRANSFER_2_AMOUNT_WITH_ADV },
    )
    .build();

    let exec_request_3 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => ACCOUNT_2_ADDR, ARG_AMOUNT => *TRANSFER_TOO_MUCH },
    )
    .build();

    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        // Exec transfer contract
        .exec(exec_request_1)
        .expect_success()
        .commit()
        // Exec transfer contract
        .exec(exec_request_2)
        .expect_success()
        .commit()
        // // Exec transfer contract
        .exec(exec_request_3)
        .commit()
        .finish();

    let error_msg = result
        .builder()
        .exec_error_message(2)
        .expect("should have error message");
    assert!(
        error_msg.contains(&format!("{:?}", ApiError::Transfer)),
        error_msg
    );
}

#[ignore]
#[test]
fn should_transfer_total_amount() {
    let mut builder = InMemoryWasmTestBuilder::default();

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => ACCOUNT_1_ADDR, "amount" => *ACCOUNT_1_INITIAL_BALANCE },
    )
    .build();

    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => ACCOUNT_2_ADDR, "amount" => *ACCOUNT_1_INITIAL_BALANCE },
    )
    .build();
    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .commit()
        .expect_success()
        .finish();
}

#[ignore]
#[test]
fn should_transfer_wasmless_account_to_purse() {
    transfer_wasmless(WasmlessTransfer::AccountMainPurseToPurse);
}

#[ignore]
#[test]
fn should_transfer_wasmless_account_to_account() {
    transfer_wasmless(WasmlessTransfer::AccountMainPurseToAccountMainPurse);
}

#[ignore]
#[test]
fn should_transfer_wasmless_account_to_account_by_key() {
    transfer_wasmless(WasmlessTransfer::AccountToAccountByKey);
}

#[ignore]
#[test]
fn should_transfer_wasmless_purse_to_purse() {
    transfer_wasmless(WasmlessTransfer::PurseToPurse);
}

#[ignore]
#[test]
fn should_transfer_wasmless_amount_as_u64() {
    transfer_wasmless(WasmlessTransfer::AmountAsU64);
}

enum WasmlessTransfer {
    AccountMainPurseToPurse,
    AccountMainPurseToAccountMainPurse,
    PurseToPurse,
    AccountToAccountByKey,
    AmountAsU64,
}

fn transfer_wasmless(wasmless_transfer: WasmlessTransfer) {
    let mut builder = init_wasmless_transform_builder();
    let transfer_amount: U512 = U512::from(1000);

    let account_1_purse = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account 1")
        .main_purse();

    let account_2_purse = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should get account 2")
        .main_purse();

    let account_1_starting_balance = builder.get_purse_balance(account_1_purse);
    let account_2_starting_balance = builder.get_purse_balance(account_2_purse);

    let runtime_args = match wasmless_transfer {
        WasmlessTransfer::AccountMainPurseToPurse => {
            runtime_args! { "target" => account_2_purse, "amount" => transfer_amount }
        }
        WasmlessTransfer::AccountMainPurseToAccountMainPurse => {
            runtime_args! { "target" => ACCOUNT_2_ADDR, "amount" => transfer_amount }
        }
        WasmlessTransfer::AccountToAccountByKey => {
            runtime_args! { "target" => types::Key::Account(ACCOUNT_2_ADDR), "amount" => transfer_amount }
        }
        WasmlessTransfer::PurseToPurse => {
            runtime_args! { "source" => account_1_purse, "target" => account_2_purse, "amount" => transfer_amount }
        }
        WasmlessTransfer::AmountAsU64 => {
            runtime_args! { "source" => account_1_purse, "target" => account_2_purse, "amount" => 1000u64 }
        }
    };

    let no_wasm_transfer_request = {
        let deploy_item = DeployItemBuilder::new()
            .with_address(ACCOUNT_1_ADDR)
            .with_empty_payment_bytes(runtime_args! {})
            .with_transfer_args(runtime_args)
            .with_authorization_keys(&[ACCOUNT_1_ADDR])
            .build();
        ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
    };

    builder
        .exec(no_wasm_transfer_request)
        .expect_success()
        .commit();

    assert_eq!(
        account_1_starting_balance - transfer_amount,
        builder.get_purse_balance(account_1_purse),
        "account 1 ending balance incorrect"
    );
    assert_eq!(
        account_2_starting_balance + transfer_amount,
        builder.get_purse_balance(account_2_purse),
        "account 2 ending balance incorrect"
    );
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_to_self_by_addr() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::TransferToSelfByAddr);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_to_self_by_key() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::TransferToSelfByKey);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_to_self_by_uref() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::TransferToSelfByURef);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_other_account_by_addr() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::OtherSourceAccountByAddr);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_other_account_by_key() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::OtherSourceAccountByKey);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_other_account_by_uref() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::OtherSourceAccountByURef);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_missing_target() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::MissingTarget);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_missing_amount() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::MissingAmount);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_source_uref_nonexistent() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::SourceURefNonexistent);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_target_uref_nonexistent() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::TargetURefNonexistent);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_invalid_source_uref() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::SourceURefNotPurse);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_invalid_target_uref() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::TargetURefNotPurse);
}

#[ignore]
#[test]
fn should_not_transfer_wasmless_other_purse_to_self_purse() {
    invalid_transfer_wasmless(InvalidWasmlessTransfer::OtherPurseToSelfPurse);
}

enum InvalidWasmlessTransfer {
    TransferToSelfByAddr,
    TransferToSelfByKey,
    TransferToSelfByURef,
    OtherSourceAccountByAddr,
    OtherSourceAccountByKey,
    OtherSourceAccountByURef,
    MissingTarget,
    MissingAmount,
    SourceURefNotPurse,
    TargetURefNotPurse,
    SourceURefNonexistent,
    TargetURefNonexistent,
    OtherPurseToSelfPurse,
}

fn invalid_transfer_wasmless(invalid_wasmless_transfer: InvalidWasmlessTransfer) {
    let mut builder = init_wasmless_transform_builder();
    let transfer_amount: U512 = U512::from(1000);

    let (addr, runtime_args, expected_error) = match invalid_wasmless_transfer {
        InvalidWasmlessTransfer::TransferToSelfByAddr => {
            // same source and target purse is invalid
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::TransferToSelfByKey => {
            // same source and target purse is invalid
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "target" => Key::Account(ACCOUNT_1_ADDR), "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::TransferToSelfByURef => {
            let account_1_purse = builder
                .get_account(ACCOUNT_1_ADDR)
                .expect("should get account 1")
                .main_purse();
            // same source and target purse is invalid
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "target" => account_1_purse, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::OtherSourceAccountByAddr => {
            // passes another account's addr as source
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "source" => ACCOUNT_2_ADDR, "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidArgument)),
            )
        }
        InvalidWasmlessTransfer::OtherSourceAccountByKey => {
            // passes another account's Key::Account as source
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "source" => Key::Account(ACCOUNT_2_ADDR), "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidArgument)),
            )
        }
        InvalidWasmlessTransfer::OtherSourceAccountByURef => {
            let account_2_purse = builder
                .get_account(ACCOUNT_2_ADDR)
                .expect("should get account 1")
                .main_purse();
            // passes another account's purse as source
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "source" => account_2_purse, "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::ForgedReference(account_2_purse)),
            )
        }
        InvalidWasmlessTransfer::MissingTarget => {
            // does not pass target
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::MissingArgument)),
            )
        }
        InvalidWasmlessTransfer::MissingAmount => {
            // does not pass amount
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "target" => ACCOUNT_2_ADDR },
                CoreError::Exec(ExecError::Revert(ApiError::MissingArgument)),
            )
        }
        InvalidWasmlessTransfer::SourceURefNotPurse => {
            let not_purse_uref =
                get_default_account_named_uref(&mut builder, TRANSFER_RESULT_NAMED_KEY);
            // passes an invalid uref as source (an existing uref that is not a purse uref)
            (
                DEFAULT_ACCOUNT_ADDR,
                runtime_args! { "source" => not_purse_uref, "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::TargetURefNotPurse => {
            let not_purse_uref =
                get_default_account_named_uref(&mut builder, TRANSFER_RESULT_NAMED_KEY);
            // passes an invalid uref as target (an existing uref that is not a purse uref)
            (
                DEFAULT_ACCOUNT_ADDR,
                runtime_args! { "target" => not_purse_uref, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::SourceURefNonexistent => {
            let nonexistent_purse = URef::new([255; 32], AccessRights::READ_ADD_WRITE);
            // passes a nonexistent uref as source; considered to be a forged reference as when
            // a caller passes a uref as source they are claiming it is a purse and that they have
            // write access to it / are allowed to take funds from it.
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "source" => nonexistent_purse, "target" => ACCOUNT_1_ADDR, "amount" => transfer_amount },
                CoreError::Exec(ExecError::ForgedReference(nonexistent_purse)),
            )
        }
        InvalidWasmlessTransfer::TargetURefNonexistent => {
            let nonexistent_purse = URef::new([255; 32], AccessRights::READ_ADD_WRITE);
            // passes a nonexistent uref as target
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "target" => nonexistent_purse, "amount" => transfer_amount },
                CoreError::Exec(ExecError::Revert(ApiError::InvalidPurse)),
            )
        }
        InvalidWasmlessTransfer::OtherPurseToSelfPurse => {
            let account_1_purse = builder
                .get_account(ACCOUNT_1_ADDR)
                .expect("should get account 1")
                .main_purse();
            let account_2_purse = builder
                .get_account(ACCOUNT_2_ADDR)
                .expect("should get account 1")
                .main_purse();

            // attempts to take from an unowned purse
            (
                ACCOUNT_1_ADDR,
                runtime_args! { "source" => account_2_purse, "target" => account_1_purse, "amount" => transfer_amount },
                CoreError::Exec(ExecError::ForgedReference(account_2_purse)),
            )
        }
    };

    let no_wasm_transfer_request = {
        let deploy_item = DeployItemBuilder::new()
            .with_address(addr)
            .with_empty_payment_bytes(runtime_args! {})
            .with_transfer_args(runtime_args)
            .with_authorization_keys(&[addr])
            .build();
        ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
    };

    builder.exec(no_wasm_transfer_request);

    let result = builder
        .get_exec_responses()
        .last()
        .expect("Expected to be called after run()")
        .get(0)
        .expect("Unable to get first deploy result");

    assert!(result.is_failure(), "was expected to fail");

    let error = result.as_error().expect("should have error");

    assert_eq!(
        format!("{}", &expected_error),
        format!("{}", error),
        "expected_error: {} actual error: {}",
        expected_error,
        error
    );
}

fn get_default_account_named_uref(builder: &mut InMemoryWasmTestBuilder, name: &str) -> URef {
    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("default account should exist");
    default_account
        .named_keys()
        .get(name)
        .expect("default account should have named key")
        .as_uref()
        .expect("should be a uref")
        .to_owned()
}

fn init_wasmless_transform_builder() -> InMemoryWasmTestBuilder {
    let mut builder = InMemoryWasmTestBuilder::default();
    let create_account_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => ACCOUNT_1_ADDR, "amount" => *DEFAULT_PAYMENT },
    )
    .build();

    let create_account_2 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => ACCOUNT_2_ADDR, "amount" => *DEFAULT_PAYMENT },
    )
    .build();

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(create_account_1)
        .expect_success()
        .commit()
        .exec(create_account_2)
        .commit()
        .expect_success()
        .to_owned()
}
