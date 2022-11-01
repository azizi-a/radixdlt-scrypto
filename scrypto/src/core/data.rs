use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::{Decode, Encode};

use crate::buffer::*;
use crate::component::{ComponentStateSubstate, KeyValueStoreEntrySubstate};
use crate::engine::{api::*, types::*, utils::*};

pub struct DataRef<V: Encode> {
    lock_handle: LockHandle,
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRef<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRef<V> {
    pub fn new(lock_handle: LockHandle, value: V) -> DataRef<V> {
        DataRef { lock_handle, value }
    }
}

impl<V: Encode> Deref for DataRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode> Drop for DataRef<V> {
    fn drop(&mut self) {
        let input = RadixEngineInput::DropLock(self.lock_handle);
        let _: () = call_engine(input);
    }
}

pub struct DataRefMut<V: Encode> {
    lock_handle: LockHandle,
    offset: SubstateOffset,
    value: V,
}

impl<V: fmt::Display + Encode> fmt::Display for DataRefMut<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<V: Encode> DataRefMut<V> {
    pub fn new(lock_handle: LockHandle, offset: SubstateOffset, value: V) -> DataRefMut<V> {
        DataRefMut {
            lock_handle,
            offset,
            value,
        }
    }
}

impl<V: Encode> Drop for DataRefMut<V> {
    fn drop(&mut self) {
        let bytes = scrypto_encode(&self.value);
        let substate = match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                scrypto_encode(&KeyValueStoreEntrySubstate(Some(bytes)))
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                scrypto_encode(&ComponentStateSubstate { raw: bytes })
            }
            s @ _ => panic!("Unsupported substate: {:?}", s),
        };
        let input = RadixEngineInput::Write(self.lock_handle, substate);
        let _: () = call_engine(input);

        let input = RadixEngineInput::DropLock(self.lock_handle);
        let _: () = call_engine(input);
    }
}

impl<V: Encode> Deref for DataRefMut<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: Encode> DerefMut for DataRefMut<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct DataPointer<V: 'static + Encode + Decode> {
    node_id: RENodeId,
    offset: SubstateOffset,
    phantom_data: PhantomData<V>,
}

impl<V: 'static + Encode + Decode> DataPointer<V> {
    pub fn new(node_id: RENodeId, offset: SubstateOffset) -> Self {
        Self {
            node_id,
            offset,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> DataRef<V> {
        let mut syscalls = Syscalls;

        let lock_handle = syscalls.sys_lock_substate(self.node_id, self.offset.clone(), false).unwrap();
        let raw_substate = syscalls.sys_read(lock_handle).unwrap();
        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = scrypto_decode(&raw_substate).unwrap();
                DataRef {
                    lock_handle,
                    value: substate,
                }
            }
        }
    }

    pub fn get_mut(&mut self) -> DataRefMut<V> {
        let mut syscalls = Syscalls;

        let lock_handle = syscalls.sys_lock_substate(self.node_id, self.offset.clone(), true).unwrap();
        let raw_substate = syscalls.sys_read(lock_handle).unwrap();

        match &self.offset {
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
                let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.0.unwrap()).unwrap(),
                }
            }
            SubstateOffset::Component(ComponentOffset::State) => {
                let substate: ComponentStateSubstate = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: scrypto_decode(&substate.raw).unwrap(),
                }
            }
            _ => {
                let substate: V = scrypto_decode(&raw_substate).unwrap();
                DataRefMut {
                    lock_handle,
                    offset: self.offset.clone(),
                    value: substate,
                }
            }
        }
    }
}
