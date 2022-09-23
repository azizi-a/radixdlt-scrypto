mod auth_zone;
mod bucket;
mod component;
mod key_value_store;
mod non_fungible;
mod package;
mod proof;
mod resource_manager;
mod system;
mod transaction_processor;
mod vault;
mod worktop;

pub use auth_zone::*;
pub use bucket::*;
pub use component::*;
pub use key_value_store::*;
pub use non_fungible::*;
pub use package::*;
pub use proof::*;
pub use resource_manager::*;
pub use system::*;
pub use transaction_processor::*;
pub use vault::*;
pub use worktop::*;

pub trait TryIntoSubstates {
    type Error;

    fn try_into_substates(self) -> Result<Vec<crate::model::Substate>, Self::Error>;
}
