use engine_shared::{gas::Gas, transform::Transform};
use log::warn;
use types::{Key, U512};

use super::Error;
use crate::engine_state::{
    execution_effect::ExecutionEffect, execution_result::ExecutionResult, op::Op,
};

fn on_fail_charge_test_helper<T>(
    f: impl Fn() -> Result<T, Error>,
    success_cost: Gas,
    error_cost: Gas,
) -> ExecutionResult {
    let _result = on_fail_charge!(f(), error_cost);
    ExecutionResult::Success {
        effect: Default::default(),
        cost: success_cost,
    }
}

#[test]
fn on_fail_charge_ok_test() {
    let val = Gas::new(U512::from(123));
    match on_fail_charge_test_helper(|| Ok(()), val, Gas::new(U512::from(456))) {
        ExecutionResult::Success { cost, .. } => assert_eq!(cost, val),
        ExecutionResult::Failure { .. } => panic!("Should be success"),
    }
}

#[test]
fn on_fail_charge_err_laziness_test() {
    let error_cost = Gas::new(U512::from(456));
    match on_fail_charge_test_helper(
        || Err(Error::GasLimit) as Result<(), _>,
        Gas::new(U512::from(123)),
        error_cost,
    ) {
        ExecutionResult::Success { .. } => panic!("Should fail"),
        ExecutionResult::Failure { cost, .. } => assert_eq!(cost, error_cost),
    }
}

#[test]
fn on_fail_charge_with_action() {
    let f = || {
        let input: Result<(), Error> = Err(Error::GasLimit);
        on_fail_charge!(input, Gas::new(U512::from(456)), {
            let mut effect = ExecutionEffect::default();

            effect.ops.insert(Key::Hash([42u8; 32]), Op::Read);
            effect
                .transforms
                .insert(Key::Hash([42u8; 32]), Transform::Identity);

            effect
        });
        ExecutionResult::Success {
            effect: Default::default(),
            cost: Gas::default(),
        }
    };
    match f() {
        ExecutionResult::Success { .. } => panic!("Should fail"),
        ExecutionResult::Failure { cost, effect, .. } => {
            assert_eq!(cost, Gas::new(U512::from(456)));
            // Check if the containers are non-empty
            assert_eq!(effect.ops.len(), 1);
            assert_eq!(effect.transforms.len(), 1);
        }
    }
}
