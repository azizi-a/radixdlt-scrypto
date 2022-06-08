use radix_engine::engine::{Receipt, TransactionExecutor};
use radix_engine::ledger::*;
use radix_engine::model::{export_abi, export_abi_by_component, extract_package, Component};
use radix_engine::wasm::DefaultWasmEngine;
use scrypto::prelude::*;
use scrypto::{abi, to_struct};
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaPrivateKey;

pub struct TestRunner {
    substate_store: InMemorySubstateStore,
    wasm_engine: DefaultWasmEngine,
    next_private_key: u64,
    next_transaction_nonce: u64,
    trace: bool,
}

impl TestRunner {
    pub fn new(trace: bool) -> Self {
        Self {
            substate_store: InMemorySubstateStore::with_bootstrap(),
            wasm_engine: DefaultWasmEngine::new(),
            next_private_key: 1, // 0 is invalid
            next_transaction_nonce: 0,
            trace,
        }
    }

    pub fn new_key_pair(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey) {
        let private_key = EcdsaPrivateKey::from_u64(self.next_private_key).unwrap();
        let public_key = private_key.public_key();

        self.next_private_key += 1;
        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (EcdsaPublicKey, EcdsaPrivateKey, NonFungibleAddress) {
        let key_pair = self.new_account();
        (
            key_pair.0,
            key_pair.1,
            NonFungibleAddress::from_public_key(&key_pair.0),
        )
    }

    pub fn new_account_with_auth_rule(&mut self, withdraw_auth: &AccessRule) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .call_method(SYSTEM_COMPONENT, "free_xrd", to_struct!())
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.new_account_with_resource(withdraw_auth, bucket_id)
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_success();

        receipt.new_component_addresses[0]
    }

    pub fn new_account(&mut self) -> (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleAddress::from_public_key(&key_pair.0)));
        let account = self.new_account_with_auth_rule(&withdraw_auth);
        (key_pair.0, key_pair.1, account)
    }

    pub fn publish_package(&mut self, name: &str) -> PackageAddress {
        self.publish_package_with_code(compile_package!(format!("./tests/{}", name)))
    }

    pub fn publish_package_with_code(&mut self, code: Vec<u8>) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .publish_package(extract_package(code).expect("Failed to extract package"))
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.new_package_addresses[0]
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        signer_public_keys: Vec<EcdsaPublicKey>,
    ) -> Receipt {
        let transaction =
            TestTransaction::new(manifest, self.next_transaction_nonce, signer_public_keys);
        self.next_transaction_nonce += 1;

        let receipt =
            TransactionExecutor::new(&mut self.substate_store, &mut self.wasm_engine, self.trace)
                .execute(&transaction);

        receipt
    }

    pub fn inspect_component(&self, component_address: ComponentAddress) -> Component {
        self.substate_store
            .get_decoded_substate(&component_address)
            .map(|(component, _)| component)
            .unwrap()
    }

    pub fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> abi::Blueprint {
        export_abi(&self.substate_store, package_address, blueprint_name)
            .expect("Failed to export ABI")
    }

    pub fn export_abi_by_component(&self, component_address: ComponentAddress) -> abi::Blueprint {
        export_abi_by_component(&self.substate_store, component_address)
            .expect("Failed to export ABI")
    }

    pub fn update_resource_auth(
        &mut self,
        function: &str,
        auth: ResourceAddress,
        token: ResourceAddress,
        set_auth: ResourceAddress,
        account: ComponentAddress,
        signer_public_key: EcdsaPublicKey,
    ) {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .create_proof_from_account(auth, account)
            .call_function(
                package,
                "ResourceCreator",
                function,
                to_struct!(token, set_auth),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        self.execute_manifest(manifest, vec![signer_public_key])
            .expect_success();
    }

    pub fn create_restricted_token(
        &mut self,
        account: ComponentAddress,
    ) -> (
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    ) {
        let mint_auth = self.create_non_fungible_resource(account);
        let burn_auth = self.create_non_fungible_resource(account);
        let withdraw_auth = self.create_non_fungible_resource(account);
        let admin_auth = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_token",
                to_struct!(mint_auth, burn_auth, withdraw_auth, admin_auth),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (
            receipt.new_resource_addresses[0],
            mint_auth,
            burn_auth,
            withdraw_auth,
            admin_auth,
        )
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_burn",
                to_struct!(auth_resource_address),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_restricted_transfer",
                to_struct![auth_resource_address],
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        (auth_resource_address, receipt.new_resource_addresses[0])
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_non_fungible_fixed",
                to_struct!(),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_success();
        receipt.new_resource_addresses[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let package = self.publish_package("resource_creator");
        let manifest = ManifestBuilder::new()
            .call_function(
                package,
                "ResourceCreator",
                "create_fungible_fixed",
                to_struct!(amount, divisibility),
            )
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.new_resource_addresses[0]
    }

    pub fn instantiate_component(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<String>,
        account: ComponentAddress,
        signer_public_key: EcdsaPublicKey,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .call_function_with_abi(
                package_address,
                blueprint_name,
                function_name,
                args,
                Some(account),
                &self.export_abi(package_address, blueprint_name),
            )
            .unwrap()
            .call_method_with_all_resources(account, "deposit_batch")
            .build();
        let receipt = self.execute_manifest(manifest, vec![signer_public_key]);
        receipt.new_component_addresses[0]
    }
}

#[macro_export]
macro_rules! assert_auth_error {
    ($error:expr) => {{
        if !matches!(
            $error,
            RuntimeError::AuthorizationError {
                authorization: _,
                function: _,
                error: ::radix_engine::model::MethodAuthorizationError::NotAuthorized
            }
        ) {
            panic!("Expected auth error but got: {:?}", $error);
        }
    }};
}

#[macro_export]
macro_rules! assert_invoke_error {
    ($result:expr, $pattern:pat) => {{
        let matches = match &$result {
            Err(radix_engine::engine::RuntimeError::InvokeError(e)) => {
                matches!(e.as_ref(), $pattern)
            }
            _ => false,
        };

        if !matches {
            panic!("Expected invoke error but got: {:?}", $result);
        }
    }};
}

pub fn wat2wasm(wat: &str) -> Vec<u8> {
    wabt::wat2wasm(
        wat.replace("${memcpy}", include_str!("wasm/snippets/memcpy.wat"))
            .replace("${memmove}", include_str!("wasm/snippets/memmove.wat"))
            .replace("${memset}", include_str!("wasm/snippets/memset.wat"))
            .replace("${buffer}", include_str!("wasm/snippets/buffer.wat")),
    )
    .expect("Failed to compiled WAT into WASM")
}
