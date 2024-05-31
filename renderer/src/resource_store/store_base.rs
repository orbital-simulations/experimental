use core::fmt;
use std::{any::Any, collections::VecDeque, fmt::Formatter, marker::PhantomData};

use super::reload_command::RebuildCommand;

pub struct StoreEntityId<T> {
    // TODO: Later on this could be reference counted and thus it could allow
    // automatic removal of the resource.
    index: usize,
    phantom_data: PhantomData<T>,
}

impl<T: Any> fmt::Display for StoreEntityId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoreEntityId<{}>({})",
            std::any::type_name::<T>(),
            self.index
        )
    }
}

impl<T: Any> fmt::Debug for StoreEntityId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoreEntityId<{}>({})",
            std::any::type_name::<T>(),
            self.index
        )
    }
}

impl<T> PartialEq for StoreEntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for StoreEntityId<T> {}

impl<T> Clone for StoreEntityId<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            phantom_data: PhantomData,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct StoreBase<T, S> {
    store: VecDeque<T>,
    store_dependants: VecDeque<VecDeque<RebuildCommand>>,
    source_data: VecDeque<S>,
    phantom_data: PhantomData<T>,
}

impl<T, S> StoreBase<T, S> {
    pub fn new() -> StoreBase<T, S> {
        StoreBase {
            store: VecDeque::new(),
            phantom_data: PhantomData,
            store_dependants: VecDeque::new(),
            source_data: VecDeque::new(),
        }
    }

    pub fn add(&mut self, entity: T, source_data: S) -> StoreEntityId<T> {
        let index = self.store.len();
        self.store.push_back(entity);
        self.store_dependants.push_back(VecDeque::new());
        self.source_data.push_back(source_data);
        StoreEntityId {
            index,
            phantom_data: PhantomData,
        }
    }

    pub fn remove(&mut self, entity_id: StoreEntityId<T>) -> T {
        self.store.remove(entity_id.index).unwrap()
    }

    pub fn register_dependant(
        &mut self,
        entity_id: &StoreEntityId<T>,
        reload_command: RebuildCommand,
    ) {
        self.store_dependants[entity_id.index].push_back(reload_command)
    }

    pub fn get_source_data(&self, entity_id: &StoreEntityId<T>) -> &S {
        &self.source_data[entity_id.index]
    }

    pub fn get_dependants(&self, entity_id: &StoreEntityId<T>) -> &VecDeque<RebuildCommand> {
        &self.store_dependants[entity_id.index]
    }

    pub fn set_entity(&mut self, id: &StoreEntityId<T>, element: T) {
        self.store[id.index] = element;
    }

    pub fn get(&self, entity_id: &StoreEntityId<T>) -> &T {
        &self.store[entity_id.index]
    }

    pub fn get_mut(&mut self, entity_id: &StoreEntityId<T>) -> &mut T {
        &mut self.store[entity_id.index]
    }
}
