use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    hash::Hash,
    ops::Deref,
    rc::{Rc, Weak},
};

pub struct Entry<C, T> {
    entry_ref: StrongReference<C, T>,
    hash: (TypeId, u64),
}

type WeakReference<C, T> = Weak<RefCell<EntryWrapper<C, T>>>;
type StrongReference<C, T> = Rc<RefCell<EntryWrapper<C, T>>>;

impl<C, T> Hash for Entry<C, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

pub trait EntryRebuilder<C> {
    fn rebuild(&self, context: &C) -> Self;
}

impl<C, T> Entry<C, T> {
    pub fn downgrade(&self) -> WeakEntry<C, T> {
        WeakEntry {
            entry_ref: Rc::downgrade(&self.entry_ref),
            hash: self.hash,
        }
    }

    pub fn rebuild(&self)
    where
        T: EntryRebuilder<C>,
    {
        let mut wrapper = self.entry_ref.borrow_mut();
        wrapper.data = {
            let context = &wrapper.internal_store.deref().borrow().store_context;
            wrapper.data.rebuild(context)
        };
        for dep in wrapper.dependant_refs.iter_mut() {
            dep.internal_rebuild();
        }
    }

    pub fn register_dep(&self, entry_ref: Box<dyn InternalEntryRebuilder>) {
        self.entry_ref.borrow_mut().dependant_refs.push(entry_ref);
    }
}

impl<C, T> Deref for Entry<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.entry_ref.as_ptr()) }
    }
}

impl<C, T> Clone for Entry<C, T> {
    fn clone(&self) -> Self {
        Entry {
            entry_ref: self.entry_ref.clone(),
            hash: self.hash,
        }
    }
}

pub trait InternalEntryRebuilder {
    fn internal_rebuild(&mut self);
}

pub struct WeakEntry<C, T> {
    entry_ref: Weak<RefCell<EntryWrapper<C, T>>>,
    hash: (TypeId, u64),
}

impl<C, T> WeakEntry<C, T> {
    pub fn upgrade(&self) -> Option<Entry<C, T>> {
        self.entry_ref.upgrade().map(|v| Entry {
            entry_ref: v,
            hash: self.hash,
        })
    }
}

impl<C, T> InternalEntryRebuilder for WeakEntry<C, T>
where
    T: EntryRebuilder<C>,
{
    fn internal_rebuild(&mut self) {
        if let Some(strong_ref) = self.upgrade() {
            let mut wrapper = strong_ref.entry_ref.deref().borrow_mut();
            wrapper.data = {
                let context = &wrapper.internal_store.deref().borrow().store_context;
                wrapper.data.rebuild(context)
            };
        }
    }
}

struct EntryWrapper<C, T> {
    label: (TypeId, u64),
    // TODO: RefCell needs to be replaced with something for thread safety.
    internal_store: Rc<RefCell<InternalStore<C, T>>>,
    data: T,
    dependant_refs: Vec<Box<dyn InternalEntryRebuilder>>,
}

impl<C, T> Deref for EntryWrapper<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<C, T> Drop for EntryWrapper<C, T> {
    fn drop(&mut self) {
        // Check if the stor estill exits.
        self.internal_store
            .deref()
            .borrow_mut()
            .store
            .remove(&self.label);
    }
}

struct InternalStore<C, T> {
    store: HashMap<(TypeId, u64), WeakReference<C, T>>,
    store_context: C,
    name: String,
}

pub struct Store<C, T> {
    // TODO: Later this will need to be something like Arc<TwLock<HashMap<..>>>
    // to prevent race conditions.
    reference: Rc<RefCell<InternalStore<C, T>>>,
}

pub trait EntryLabel {
    fn unique_label(&self) -> (TypeId, u64);
}

impl<C, T> Store<C, T> {
    pub fn new(store_context: C, name: String) -> Self {
        Self {
            reference: Rc::new(RefCell::new(InternalStore {
                store_context,
                store: HashMap::new(),
                name,
            })),
        }
    }

    fn create_entry<L>(&self, data: T, label: &L) -> (StrongReference<C, T>, WeakReference<C, T>)
    where
        L: EntryLabel,
    {
        let strong_ref = Rc::new(RefCell::new(EntryWrapper {
            internal_store: self.reference.clone(),
            label: label.unique_label(),
            data,
            dependant_refs: vec![],
        }));
        let weak_ref = Rc::downgrade(&strong_ref);
        (strong_ref, weak_ref)
    }

    pub fn get_entry<F, F2, L, M>(&mut self, label: &L, constructor: F, after: F2) -> Entry<C, T>
    where
        L: EntryLabel,
        F: FnOnce(&mut C) -> (T, M),
        F2: FnOnce(&mut C, &Entry<C, T>, M),
    {
        let mut internal_store = self.reference.deref().borrow_mut();
        println!(
            "store size for storer: {}, len: {}",
            internal_store.name,
            internal_store.store.len()
        );
        let possible_ref = internal_store.store.get(&label.unique_label());
        let hash = label.unique_label();
        match possible_ref.as_ref() {
            Some(weak_ref) => match weak_ref.upgrade() {
                None => {
                    let (data, meta_data) = constructor(&mut internal_store.store_context);
                    let (strong_ref, weak_ref) = self.create_entry(data, label);
                    internal_store.store.insert(label.unique_label(), weak_ref);
                    let entry = Entry {
                        hash,
                        entry_ref: strong_ref,
                    };
                    after(&mut internal_store.store_context, &entry, meta_data);
                    entry
                }
                Some(strong_ref) => Entry {
                    entry_ref: strong_ref,
                    hash,
                },
            },
            None => {
                let (data, meta_data) = constructor(&mut internal_store.store_context);
                let (strong_ref, weak_ref) = self.create_entry(data, label);
                internal_store.store.insert(label.unique_label(), weak_ref);
                let entry = Entry {
                    entry_ref: strong_ref,
                    hash,
                };
                after(&mut internal_store.store_context, &entry, meta_data);
                entry
            }
        }
    }

    pub fn get_context_mut(&mut self) -> RefMut<'_, C> {
        std::cell::RefMut::<'_, InternalStore<C, T>>::map(
            self.reference.deref().borrow_mut(),
            |r| &mut r.store_context,
        )
    }

    pub fn get_context(&self) -> Ref<'_, C> {
        std::cell::Ref::<'_, InternalStore<C, T>>::map(self.reference.deref().borrow(), |r| {
            &r.store_context
        })
    }
}
