use core::fmt;
use std::{any::Any, collections::VecDeque, fmt::Formatter, marker::PhantomData};

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
pub struct StoreBase<T> {
    store: VecDeque<T>,
    phantom_data: PhantomData<T>,
}

impl<T> StoreBase<T> {
    pub fn new() -> StoreBase<T> {
        StoreBase {
            store: VecDeque::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn add(&mut self, entity: T) -> StoreEntityId<T> {
        let index = self.store.len();
        self.store.push_back(entity);
        StoreEntityId {
            index,
            phantom_data: PhantomData,
        }
    }

    pub fn remove(&mut self, entity_id: StoreEntityId<T>) -> T {
        self.store.remove(entity_id.index).unwrap()
    }

    pub fn get(&self, entity_id: &StoreEntityId<T>) -> &T {
        &self.store[entity_id.index]
    }

    pub fn get_mut(&mut self, entity_id: &StoreEntityId<T>) -> &mut T {
        &mut self.store[entity_id.index]
    }
}
