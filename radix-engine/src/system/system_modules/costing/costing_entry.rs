use super::FeeTable;
use crate::kernel::actor::Actor;
use crate::track::interface::{StoreAccess, StoreAccessInfo, StoreCommit};
use crate::types::*;
use radix_engine_interface::*;

#[derive(Debug, IntoStaticStr)]
pub enum CostingEntry<'a> {
    /* TX */
    TxBaseCost,
    TxPayloadCost {
        size: usize,
    },
    TxSignatureVerification {
        num_signatures: usize,
    },

    /* execution */
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        input_size: usize,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        wasm_execution_units: u32,
    },
    PrepareWasmCode {
        size: usize,
    },

    /* invoke */
    BeforeInvoke {
        actor: &'a Actor,
        input_size: usize,
    },
    AfterInvoke {
        output_size: usize,
    },

    /* node */
    AllocateNodeId,
    CreateNode {
        node_id: &'a NodeId,
        total_substate_size: usize,
        store_access: &'a StoreAccessInfo,
    },
    DropNode {
        total_substate_size: usize,
    },
    MoveModules,
    OpenSubstate {
        node_id: &'a NodeId,
        value_size: usize,
    },
    ReadSubstate {
        value_size: usize,
    },
    WriteSubstate {
        value_size: usize,
    },
    CloseSubstate,


    /* unstable node apis */
    SetSubstate {
        value_size: usize,
    },
    RemoveSubstateBase,
    ScanSortedSubstatesBase,
    ScanSubstatesBase,
    TakeSubstatesBase,
    StoreAccess {
        store_access: &'a StoreAccess,
    },

    /* commit */
    Commit {
        store_commit: &'a StoreCommit,
    },

    /* system */
    LockFee,
    QueryFeeReserve,
    QueryActor,
    QueryAuthZone,
    AssertAccessRule,
    QueryTransactionHash,
    GenerateRuid,
    EmitEvent {
        size: usize,
    },
    EmitLog {
        size: usize,
    },
    Panic {
        size: usize,
    },

    /* system modules */
    RoyaltyModule {
        direct_charge: u32,
    },
    AuthModule {
        direct_charge: u32,
    },
}

impl<'a> CostingEntry<'a> {
    pub fn to_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            CostingEntry::TxBaseCost => ft.tx_base_cost(),
            CostingEntry::TxPayloadCost { size } => ft.tx_payload_cost(*size),
            CostingEntry::TxSignatureVerification { num_signatures } => {
                ft.tx_signature_verification_cost(*num_signatures)
            }
            CostingEntry::RunNativeCode {
                package_address,
                export_name,
                input_size,
            } => ft.run_native_code_cost(package_address, export_name, input_size),
            CostingEntry::RunWasmCode {
                package_address,
                export_name,
                wasm_execution_units,
            } => ft.run_wasm_code_cost(package_address, export_name, *wasm_execution_units),
            CostingEntry::PrepareWasmCode { size } => ft.instantiate_wasm_code_cost(*size),
            CostingEntry::BeforeInvoke { actor, input_size } => {
                ft.before_invoke_cost(actor, *input_size)
            }
            CostingEntry::AfterInvoke { output_size } => ft.after_invoke_cost(*output_size),
            CostingEntry::AllocateNodeId => ft.allocate_node_id_cost(),
            CostingEntry::CreateNode {
                node_id,
                total_substate_size,
                store_access,
            } => ft.create_node_cost(node_id, *total_substate_size, store_access),
            CostingEntry::DropNode {
                total_substate_size,
            } => ft.drop_node_cost(*total_substate_size),
            CostingEntry::MoveModules => ft.move_modules_cost(),
            CostingEntry::OpenSubstate {
                node_id: _,
                value_size,
            } => ft.open_substate_cost(*value_size),
            CostingEntry::ReadSubstate {
                value_size,
            } => ft.read_substate_cost(*value_size),
            CostingEntry::WriteSubstate {
                value_size,
            } => ft.write_substate_cost(*value_size),
            CostingEntry::CloseSubstate => ft.close_substate_cost(),
            CostingEntry::SetSubstate {
                value_size,
            } => ft.set_substate_cost(*value_size),
            CostingEntry::RemoveSubstateBase => ft.remove_substate_base_cost(),
            CostingEntry::ScanSubstatesBase => ft.scan_substates_base_cost(),
            CostingEntry::ScanSortedSubstatesBase => ft.scan_sorted_substates_base_cost(),
            CostingEntry::TakeSubstatesBase => ft.take_substates_base_cost(),
            CostingEntry::StoreAccess { store_access } => ft.store_access_cost(store_access),
            CostingEntry::Commit { store_commit } => ft.store_commit_cost(store_commit),
            CostingEntry::LockFee => ft.lock_fee_cost(),
            CostingEntry::QueryFeeReserve => ft.query_fee_reserve_cost(),
            CostingEntry::QueryActor => ft.query_actor_cost(),
            CostingEntry::QueryAuthZone => ft.query_auth_zone_cost(),
            CostingEntry::AssertAccessRule => ft.assert_access_rule_cost(),
            CostingEntry::QueryTransactionHash => ft.query_transaction_hash_cost(),
            CostingEntry::GenerateRuid => ft.generate_ruid_cost(),
            CostingEntry::EmitEvent { size } => ft.emit_event_cost(*size),
            CostingEntry::EmitLog { size } => ft.emit_log_cost(*size),
            CostingEntry::Panic { size } => ft.panic_cost(*size),
            CostingEntry::RoyaltyModule { direct_charge } => *direct_charge,
            CostingEntry::AuthModule { direct_charge } => *direct_charge,
        }
    }
}

impl<'a> CostingEntry<'a> {
    pub fn to_trace_key(&self) -> String {
        match self {
            CostingEntry::RunNativeCode { export_name, .. } => {
                format!("RunNativeCode::{}", export_name)
            }
            CostingEntry::RunWasmCode { export_name, .. } => {
                format!("RunWasmCode::{}", export_name)
            }
            CostingEntry::OpenSubstate { node_id, .. } => {
                format!(
                    "OpenSubstate::{}",
                    node_id.entity_type().map(|x| x.into()).unwrap_or("?")
                )
            }
            CostingEntry::Commit { store_commit } => {
                format!(
                    "Commit::{}",
                    store_commit
                        .node_id()
                        .entity_type()
                        .map(|x| x.into())
                        .unwrap_or("?")
                )
            }
            x => Into::<&'static str>::into(x).to_string(),
        }
    }
}
