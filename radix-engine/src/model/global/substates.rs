use crate::types::*;
use radix_engine_interface::api::types::{PackageId, RENodeId, ResourceManagerId, SystemId};

// TODO: clean up after `Owned(RENodeId)`?
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    Resource(ResourceManagerId),
    Package(PackageId),
    System(SystemId),
}

impl GlobalAddressSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            GlobalAddressSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalAddressSubstate::Package(id) => RENodeId::Package(*id),
            GlobalAddressSubstate::System(SystemId::EpochManager(id)) => {
                RENodeId::EpochManager(*id)
            }
            GlobalAddressSubstate::System(SystemId::Clock(id)) => RENodeId::Clock(*id),
        }
    }
}
