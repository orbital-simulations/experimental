use std::{
    any::Any, collections::{HashMap, HashSet}, hash::Hash, marker::PhantomData, ptr, rc::Rc, sync::{Arc, Mutex, MutexGuard}
};

#[derive(Debug)]
pub struct StoreID<T> {
    index: usize,
    phantom_t: PhantomData<T>,
}

impl<T> Clone for StoreID<T> {
    fn clone(&self) -> Self {
        StoreID::new(&self.index)
    }
}

impl<T> StoreID<T> {
    fn new(index: &usize) -> StoreID<T> {
        StoreID {
            index: *index,
            phantom_t: PhantomData,
        }
    }
}

impl<T> Eq for StoreID<T> {}

impl<T> PartialEq for StoreID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Hash for StoreID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

pub struct Store<T>
where
    T: StorableResource,
{
    locked_store: Arc<Mutex<InnerStore<T>>>,
}

impl<T> Clone for Store<T>
where
    T: StorableResource,
{
    fn clone(&self) -> Self {
        Self {
            locked_store: self.locked_store.clone(),
        }
    }
}

impl<T> Store<T>
where
    T: StorableResource + 'static,
{
    pub fn new(context: T::Context) -> Self {
        Self {
            locked_store: Arc::new(Mutex::new(InnerStore::new(context))),
        }
    }

    pub fn lock(&self) -> UnlockedStore<'_, T> {
        // TODO: Think about propagating the locking error...
        UnlockedStore::new(&self.locked_store, self.locked_store.lock().unwrap())
    }
}

pub struct InnerStore<T>
where
    T: StorableResource,
{
    index_cache: HashMap<T::Description, usize>,
    store: Vec<T>,
    context: T::Context,
    dependants: HashMap<StoreID<T>, HashSet<RebuildableHashable>>,
}

impl<T> InnerStore<T>
where
    T: StorableResource,
{
    pub fn new(context: T::Context) -> InnerStore<T> {
        InnerStore {
            index_cache: HashMap::new(),
            store: Vec::new(),
            context,
            dependants: HashMap::new(),
        }
    }
}

pub struct UnlockedStore<'a, T>
where
    T: StorableResource,
{
    store_ref: &'a Arc<Mutex<InnerStore<T>>>,
    inner: MutexGuard<'a, InnerStore<T>>,
}

impl<'a, T> UnlockedStore<'a, T>
where
    T: StorableResource + 'static + Any,
{
    fn new(store_ref: &'a Arc<Mutex<InnerStore<T>>>, inner: MutexGuard<'a, InnerStore<T>>) -> Self {
        Self { store_ref, inner }
    }
    pub fn get_or_create(&mut self, description: &T::Description) -> StoreID<T> {
        if let Some(index) = self.inner.index_cache.get(description) {
            StoreID::new(index)
        } else {
            let index = self.inner.store.len();
            let fat_id = FatStoreID {
                store: Store {
                    locked_store: self.store_ref.clone(),
                },
                id: StoreID::new(&index),
            };
            let data = T::build(&self.inner.context, description);
            T::register_dependences(&self.inner.context, description, fat_id);
            self.inner.store.push(data);
            self.inner.index_cache.insert(description.clone(), index);
            StoreID::new(&index)
        }
    }

    pub fn rebuild(&mut self, id: &StoreID<T>) {
        let mut description = None;
        for (description_, index) in self.inner.index_cache.iter() {
            if id.index == *index {
                description = Some(description_);
            }
        }

        if let Some(description) = description {
            let data = T::build(&self.inner.context, description);
            self.inner.store[id.index] = data;
        }

    }

    pub fn get_fat_id(&self, id: StoreID<T>) -> FatStoreID<T> {
        FatStoreID {
            store: Store {
                locked_store: self.store_ref.clone(),
            },
            id,
        }
    }

    pub fn get_ref(&self, id: &StoreID<T>) -> &T {
        &self.inner.store[id.index]
    }

    pub fn add_dependant(&mut self, dependant: Rc<dyn RebuildableEntry>, id: &StoreID<T>) {
        let untyped_dependant = RebuildableHashable::new(dependant);
        if let Some(entry) = self.inner.dependants.get_mut(id) {
            entry.insert(untyped_dependant);
        } else {
            let mut new_set = HashSet::new();
            new_set.insert(untyped_dependant);
            self.inner.dependants.insert(id.clone(), new_set);
        };
    }
}

#[derive(Clone)]
pub struct RebuildableHashable {
    wrapped_value: Rc<dyn RebuildableEntry>,
}

impl RebuildableHashable {
    fn new(v: Rc<dyn RebuildableEntry>) -> Self {
        Self { wrapped_value: v }
    }
}

impl Eq for RebuildableHashable {}

impl PartialEq for RebuildableHashable {
    fn eq(&self, other: &Self) -> bool {
        self.wrapped_value.index() == other.wrapped_value.index()
            && ptr::eq(
                self.wrapped_value.store_ptr(),
                other.wrapped_value.store_ptr(),
            )
    }
}

impl Hash for RebuildableHashable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.wrapped_value.index().hash(state);
        ptr::hash(self.wrapped_value.store_ptr(), state);
    }
}

pub struct FatStoreID<T>
where
    T: StorableResource,
{
    store: Store<T>,
    id: StoreID<T>,
}

impl<T> Eq for FatStoreID<T> where T: StorableResource {}

impl<T> PartialEq for FatStoreID<T>
where
    T: StorableResource,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.store.locked_store, &other.store.locked_store)
    }
}

impl<T> Hash for FatStoreID<T>
where
    T: StorableResource,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        ptr::hash(Arc::as_ptr(&self.store.locked_store), state);
    }
}

impl<T> Clone for FatStoreID<T>
where
    T: StorableResource,
{
    fn clone(&self) -> FatStoreID<T> {
        Self {
            store: self.store.clone(),
            id: self.id.clone(),
        }
    }
}

pub trait StorableResource: Sized {
    type Context;
    type Description: Eq + PartialEq + Hash + Clone;
    fn build(context: &Self::Context, description: &Self::Description) -> Self;
    fn register_dependences(
        context: &Self::Context,
        description: &Self::Description,
        fat_id: FatStoreID<Self>,
    );
}

pub struct Dummy {}

pub trait RebuildableEntry {
    fn rebuild(&self);
    fn store_ptr(&self) -> *const Dummy;
    fn index(&self) -> usize;
}

impl<T> RebuildableEntry for FatStoreID<T>
where
    T: StorableResource + 'static,
{
    fn rebuild(&self) {
        let dependants: HashSet<RebuildableHashable> = {
            let mut locked_store = self.store.lock();
            locked_store.rebuild(&self.id);
            match locked_store.inner.dependants.get(&self.id) {
                Some(v) => v.clone(),
                None => HashSet::new(),
            }
        };
        for dependant in dependants {
            dependant.wrapped_value.rebuild();
        }
    }

    fn index(&self) -> usize {
        self.id.index
    }

    fn store_ptr(&self) -> *const Dummy {
        Arc::as_ptr(&self.store.locked_store) as *const Dummy
    }
}
