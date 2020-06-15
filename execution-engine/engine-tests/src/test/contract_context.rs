use assert_matches::assert_matches;
use engine_core::{engine_state::Error, execution};
use engine_test_support::{
    internal::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    Code, SessionBuilder, TestContextBuilder, Hash, DEFAULT_ACCOUNT_ADDR,
};
use types::{contracts::CONTRACT_INITIAL_VERSION, runtime_args, Key, RuntimeArgs, account::PublicKey, U512};

const CONTRACT_HEADERS: &str = "contract_context.wasm";
const PACKAGE_HASH_KEY: &str = "package_hash_key";
const PACKAGE_ACCESS_KEY: &str = "package_access_key";
const SESSION_CODE_TEST: &str = "session_code_test";
const CONTRACT_CODE_TEST: &str = "contract_code_test";
const ADD_NEW_KEY_AS_SESSION: &str = "add_new_key_as_session";
const NEW_KEY: &str = "new_key";
const SESSION_CODE_CALLER_AS_CONTRACT: &str = "session_code_caller_as_contract";
const ARG_AMOUNT: &str = "amount";

#[ignore]
#[test]
fn should_enforce_intended_execution_contexts() {
    // This test runs a contract that's after every call extends the same key with
    // more data
    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_HEADERS,
        RuntimeArgs::default(),
    )
    .build();

    let exec_request_2 = {
        let args = runtime_args! {};
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_stored_versioned_contract_by_name(
                PACKAGE_HASH_KEY,
                Some(CONTRACT_INITIAL_VERSION),
                SESSION_CODE_TEST,
                args,
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
            .with_deploy_hash([3; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let exec_request_3 = {
        let args = runtime_args! {};
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_stored_versioned_contract_by_name(
                PACKAGE_HASH_KEY,
                Some(CONTRACT_INITIAL_VERSION),
                CONTRACT_CODE_TEST,
                args,
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
            .with_deploy_hash([3; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let exec_request_4 = {
        let args = runtime_args! {};
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_stored_versioned_contract_by_name(
                PACKAGE_HASH_KEY,
                Some(CONTRACT_INITIAL_VERSION),
                ADD_NEW_KEY_AS_SESSION,
                args,
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
            .with_deploy_hash([4; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };
    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder.exec(exec_request_1).expect_success().commit();

    builder.exec(exec_request_2).expect_success().commit();

    builder.exec(exec_request_3).expect_success().commit();

    builder.exec(exec_request_4).expect_success().commit();

    let account = builder
        .query(None, Key::Account(DEFAULT_ACCOUNT_ADDR), &[])
        .expect("should query account")
        .as_account()
        .cloned()
        .expect("should be account");

    let _package_hash = account
        .named_keys()
        .get(PACKAGE_HASH_KEY)
        .expect("should have contract package");
    let _access_uref = account
        .named_keys()
        .get(PACKAGE_ACCESS_KEY)
        .expect("should have package hash");

    let account = builder
        .query(None, Key::Account(DEFAULT_ACCOUNT_ADDR), &[])
        .expect("should query account")
        .as_account()
        .cloned()
        .expect("should be account");

    let _new_key = account
        .named_keys()
        .get(NEW_KEY)
        .expect("new key should be there");
}

#[ignore]
#[test]
fn should_not_call_session_from_contract() {
    // This test runs a contract that's after every call extends the same key with
    // more data
    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_HEADERS,
        RuntimeArgs::default(),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder.exec(exec_request_1).expect_success().commit();

    let account = builder
        .query(None, Key::Account(DEFAULT_ACCOUNT_ADDR), &[])
        .expect("should query account")
        .as_account()
        .cloned()
        .expect("should be account");

    let contract_package_hash = account
        .named_keys()
        .get(PACKAGE_HASH_KEY)
        .cloned()
        .expect("should have contract package");

    let exec_request_2 = {
        let args = runtime_args! {
            PACKAGE_HASH_KEY => contract_package_hash,
        };
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_stored_versioned_contract_by_name(
                PACKAGE_HASH_KEY,
                Some(CONTRACT_INITIAL_VERSION),
                SESSION_CODE_CALLER_AS_CONTRACT,
                args,
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
            .with_deploy_hash([3; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    builder.exec(exec_request_2).commit();

    let response = builder
        .get_exec_responses()
        .last()
        .expect("should have last response");
    assert_eq!(response.len(), 1);
    let exec_response = response.last().expect("should have response");
    let error = exec_response.as_error().expect("should have error");
    assert_matches!(error, Error::Exec(execution::Error::InvalidContext));
}

#[ignore]
#[test]
fn should_keep_context() {
    const CONTRACT: &str = "contract";
    const CONTRACT_HASH: &str = "contract_hash";
    const COUNTER: &str = "counter";
    const INCREMENT: &str = "increment";
    const WASM: &str = "contract_context_2.wasm";

    // Prepare the context.
    let mut context = TestContextBuilder::new()
            .with_account(DEFAULT_ACCOUNT_ADDR, U512::from(128_000_000))
            .build();

    // Deploy the contract_context_2.wasm.
    let session_code = Code::from(WASM);
    let session_args = runtime_args! {};
    let session = SessionBuilder::new(session_code, session_args)
        .with_address(DEFAULT_ACCOUNT_ADDR)
        .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
        .build();
    context.run(session);

    // Read the contract hash.
    let contract_hash: Hash = context
        .query(DEFAULT_ACCOUNT_ADDR, &[CONTRACT_HASH])
        .unwrap_or_else(|_| panic!("{} contract not found", CONTRACT_HASH))
        .into_t()
        .unwrap_or_else(|_| panic!("{} has wrong type", CONTRACT_HASH));

    // Assert the counter value.
    // The previous deploy should have created a new contract with counter initialized to 10,
    // and then should have call "increment", so the value should be 11. 
    let counter: u64 = context.query(DEFAULT_ACCOUNT_ADDR, &[CONTRACT, COUNTER]).unwrap().into_t().unwrap();
    assert_eq!(counter, 11);

    // Call the increment method.
    let session_code = Code::Hash(contract_hash, INCREMENT.to_string());
    let session_args = runtime_args! {};
    let session = SessionBuilder::new(session_code, session_args)
        .with_address(DEFAULT_ACCOUNT_ADDR)
        .with_authorization_keys(&[DEFAULT_ACCOUNT_ADDR])
        .build();
    context.run(session);

    // The counter value should be 12 now.
    let counter: u64 = context.query(DEFAULT_ACCOUNT_ADDR, &[CONTRACT, COUNTER]).unwrap().into_t().unwrap();
    assert_eq!(counter, 12);    
}