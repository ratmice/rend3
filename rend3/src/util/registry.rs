use fnv::FnvBuildHasher;
use indexmap::map::IndexMap;
use rend3_types::ResourceHandle;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Weak,
    },
};

#[derive(Debug)]
struct ResourceStorage<T> {
    refcount: Weak<()>,
    data: T,
}

#[derive(Debug)]
pub struct ResourceRegistry<T, HandleType> {
    mapping: IndexMap<usize, ResourceStorage<T>, FnvBuildHasher>,
    current_idx: AtomicUsize,
    _phantom: PhantomData<HandleType>,
}
impl<T, HandleType> ResourceRegistry<T, HandleType> {
    pub fn new() -> Self {
        Self {
            mapping: IndexMap::with_hasher(FnvBuildHasher::default()),
            current_idx: AtomicUsize::new(0),
            _phantom: PhantomData,
        }
    }

    pub fn allocate(&self) -> ResourceHandle<HandleType> {
        let idx = self.current_idx.fetch_add(1, Ordering::Relaxed);

        ResourceHandle::new(idx)
    }

    pub fn insert(&mut self, handle: &ResourceHandle<HandleType>, data: T) -> usize {
        self.mapping
            .insert_full(
                handle.get(),
                ResourceStorage {
                    refcount: handle.get_weak_refcount(),
                    data,
                },
            )
            .0
    }

    pub fn remove_all_dead(&mut self, mut func: impl FnMut(&mut Self, usize, T)) {
        for idx in (0..self.mapping.len()).rev() {
            let element = self.mapping.get_index(idx).unwrap().1;
            if element.refcount.strong_count() == 0 {
                let (_, value) = self.mapping.swap_remove_index(idx).unwrap();
                func(self, idx, value.data)
            }
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&usize, &T)> + Clone {
        self.mapping
            .iter()
            .map(|(idx, ResourceStorage { data, .. })| (idx, data))
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &T> + Clone {
        self.mapping.values().map(|ResourceStorage { data, .. }| data)
    }

    pub fn values_mut(&mut self) -> impl ExactSizeIterator<Item = &mut T> {
        self.mapping.values_mut().map(|ResourceStorage { data, .. }| data)
    }

    pub fn get(&self, handle: &ResourceHandle<HandleType>) -> &T {
        &self.mapping.get(&handle.get()).unwrap().data
    }

    pub fn get_raw(&self, handle: &usize) -> &T {
        &self.mapping.get(handle).unwrap().data
    }

    pub fn get_mut(&mut self, handle: &ResourceHandle<HandleType>) -> &mut T {
        &mut self.mapping.get_mut(&handle.get()).unwrap().data
    }

    pub fn get_index_of(&self, handle: &ResourceHandle<HandleType>) -> usize {
        self.mapping.get_index_of(&handle.get()).unwrap()
    }

    pub fn count(&self) -> usize {
        self.mapping.len()
    }
}

impl<T, HandleType> Default for ResourceRegistry<T, HandleType> {
    fn default() -> Self {
        Self::new()
    }
}
