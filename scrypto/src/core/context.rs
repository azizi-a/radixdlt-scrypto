use crate::core::*;
use crate::crypto::*;
use crate::engine::{api::*, call_engine};
use crate::rust::borrow::ToOwned;
use crate::rust::vec::Vec;

/// A utility for accessing runtime.
#[derive(Debug)]
pub struct Context {}

impl Context {
    /// Returns the running entity, a component if within a call-method context or a
    /// blueprint if within a call-function context.
    pub fn actor() -> Actor {
        let input = GetActorInput {};
        let output: GetActorOutput = call_engine(GET_ACTOR, input);
        output.actor
    }

    /// Returns the package.
    pub fn package() -> PackageRef {
        match Context::actor() {
            Actor::Blueprint(package, _) => package,
            Actor::Component(component) => component.blueprint().0,
        }
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_engine(GET_TRANSACTION_HASH, input);
        output.transaction_hash
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = GetCurrentEpochInput {};
        let output: GetCurrentEpochOutput = call_engine(GET_CURRENT_EPOCH, input);
        output.current_epoch
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let input = GenerateUuidInput {};
        let output: GenerateUuidOutput = call_engine(GENERATE_UUID, input);

        output.uuid
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S: AsRef<str>>(
        package_ref: PackageRef,
        blueprint_name: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> Vec<u8> {
        let input = CallFunctionInput {
            package_ref,
            blueprint_name: blueprint_name.as_ref().to_owned(),
            function: function.as_ref().to_owned(),
            args,
        };
        let output: CallFunctionOutput = call_engine(CALL_FUNCTION, input);

        output.rtn
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>>(
        component_ref: ComponentRef,
        method: S,
        args: Vec<Vec<u8>>,
    ) -> Vec<u8> {
        let input = CallMethodInput {
            component_ref: component_ref,
            method: method.as_ref().to_owned(),
            args,
        };
        let output: CallMethodOutput = call_engine(CALL_METHOD, input);

        output.rtn
    }
}
