use crate::static_storage::StaticStorage;

/// A Component of an Entity
/// 
/// Components are one of the foundational building blocks of an Entity Component System. Components are analagous to
/// fields of a struct, however they differ in how they are accessed and stored. All Components of the same type are
/// stored in the same data structure. This means that if you want to access a Component of an Entity, you would need
/// to search the datastructure for the corrisbonding Component. This has it's trade offs. On the one hand accessing the Component of an Entity
/// is much slower than accessing the analogous field of a struct. However this allows for the ability to create Entities with any combination
/// of Components at runtime, it also allows for the ability to join entities that all have Components in common. 
pub trait Component<S> : StaticStorage<S> where S : ComponentStorage<Self> + 'static {}

/// The ComponentStorage trait represents a data structure that stores Components 
/// Every ComponentStorage is different but Idealy ComponentStorages should be able
/// to look up components based on an EntityID and produce an iterator that moves through
/// each component sorted by EntityID.
pub trait ComponentStorage<V> : Default {
    type EntityID;
    fn component_insert(&mut self, key : Self::EntityID, value : V);
    fn component_remove(&mut self, key : &Self::EntityID);
}

/// A basic implementation for ComponentStorage on the standard BTreeMap
impl<K: Ord + Copy, V> ComponentStorage<V> for std::collections::BTreeMap<K, V> {
    type EntityID = K;
    fn component_insert(&mut self, key : Self::EntityID, value : V){ self.insert(key, value); }
    fn component_remove(&mut self, key : &Self::EntityID) { self.remove(key); }
}

/// impl_component! macro implements the Component Trait on a given struct
/// The first argument is the struct that Component will be implemented on
/// The second argument is the ComponentStorage that will store the Component.
/// 
/// For example impl_component!((), BTreeMap<u64, ()>) will create a storage for the
/// () type, and the EntityId's will be of type u64.
#[macro_export] macro_rules! impl_component {
    ($name:ty, $storage:ty) => {
        impl genecs::static_storage::StaticStorage<$storage> for $name {
            unsafe fn get_static() -> &'static mut ($storage, (u64, u64)) {
                // Import std::sync::Once and std::sync::ONCE_INIT as the user might not import themselves
                use std::sync::{Once, ONCE_INIT};
                // Use an Inner type to call ::defualt as types with generic arguments will cause an error
                type ComponentStorageInner = $storage;
                // Initialize the static storage to a const (Option, Once)
                static mut STATIC_STORAGE : (Option<($storage, (u64, u64))>, Once) = (None, ONCE_INIT);
                // At runtime Change the static storages None to a Some($storage, (u64, u64))
                STATIC_STORAGE.1.call_once(|| {
                    STATIC_STORAGE.0 = Some((ComponentStorageInner::default(), (0,0)));
                });
                // Coerce a mutable refrence into a mutable pointer
                let ptr = &mut STATIC_STORAGE.0 as * mut Option<($storage, (u64, u64))>;
                // Unwrap the option and return the inner storage
                match *ptr {
                    Some(ref mut x) => x,
                    None => unreachable!(),
                }
            }
        }
        impl Component<$storage> for $name {}
    }
}
