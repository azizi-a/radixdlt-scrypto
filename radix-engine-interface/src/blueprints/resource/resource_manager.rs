use crate::address::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::ScryptoCustomValueKind;
use crate::math::*;
use crate::well_known_scrypto_custom_type;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto_abi::*;
use utils::{copy_u8_array, ContextualDisplay};

use crate::*;

pub struct ResourceManagerAbi;

impl ResourceManagerAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, LegacyDescribe)]
pub enum VaultMethodAuthKey {
    Withdraw,
    Deposit,
    Recall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, LegacyDescribe)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    UpdateNonFungibleData,
    UpdateMetadata,
    Withdraw,
    Deposit,
    Recall,
}

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT: &str = "create_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateFungibleInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
}

pub const RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT: &str =
    "create_fungible_with_initial_supply_and_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateFungibleWithInitialSupplyAndAddressInput {
    pub divisibility: u8,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub initial_supply: Decimal,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT: &str = "create_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateNonFungibleInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_non_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateNonFungibleWithInitialSupplyInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT: &str =
    "create_non_fungible_with_address";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateNonFungibleWithAddressInput {
    pub id_type: NonFungibleIdType,
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub resource_address: [u8; 26], // TODO: Clean this up
}

pub const RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY: &str =
    "create_uuid_non_fungible_with_initial_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateUuidNonFungibleWithInitialSupplyInput {
    pub metadata: BTreeMap<String, String>,
    pub access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    pub entries: BTreeSet<(Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_BURN_BUCKET_IDENT: &str = "burn_bucket";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnBucketInput {
    pub bucket: Bucket,
}

impl Clone for ResourceManagerBurnBucketInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const RESOURCE_MANAGER_BURN_IDENT: &str = "burn";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnInput {
    pub bucket: Bucket,
}

pub const RESOURCE_MANAGER_CREATE_VAULT_IDENT: &str = "create_vault";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateVaultInput {}

pub const RESOURCE_MANAGER_CREATE_BUCKET_IDENT: &str = "create_bucket";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateBucketInput {}

pub const RESOURCE_MANAGER_UPDATE_VAULT_AUTH_IDENT: &str = "update_vault_auth";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerUpdateVaultAuthInput {
    pub method: VaultMethodAuthKey,
    pub access_rule: AccessRule,
}

pub const RESOURCE_MANAGER_SET_VAULT_AUTH_MUTABILITY_IDENT: &str = "set_vault_auth_mutability";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerSetVaultAuthMutabilityInput {
    pub method: VaultMethodAuthKey,
    pub mutability: AccessRule,
}

pub const RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerUpdateNonFungibleDataInput {
    pub id: NonFungibleLocalId,
    pub data: Vec<u8>,
}

pub const RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerNonFungibleExistsInput {
    pub id: NonFungibleLocalId,
}

pub const RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub const RESOURCE_MANAGER_MINT_NON_FUNGIBLE: &str = "mint_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintNonFungibleInput {
    pub entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE: &str = "mint_uuid_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintUuidNonFungibleInput {
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
}

pub const RESOURCE_MANAGER_MINT_FUNGIBLE: &str = "mint_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerMintFungibleInput {
    pub amount: Decimal,
}

pub const RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT: &str = "get_resource_type";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetResourceTypeInput {}

pub const RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT: &str = "get_total_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetTotalSupplyInput {}

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAddress {
    Normal([u8; 26]),
}

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Resource => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn raw(&self) -> [u8; 26] {
        match self {
            Self::Normal(v) => v.clone(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

well_known_scrypto_custom_type!(
    ResourceAddress,
    ScryptoCustomValueKind::ResourceAddress,
    Type::ResourceAddress,
    27,
    RESOURCE_ADDRESS_ID
);

//======
// text
//======

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Normal(_) => {
                write!(f, "NormalResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}