use crate::static_storage::StaticStorage;

pub trait Resource : StaticStorage<Self> + 'static {}

/// impl_resource! macro implements the Component Trait on a given struct
/// The first argument is the struct that Component will be implemented on
/// The second argument is the ComponentStorage that will store the Component.
/// 
/// For example impl_resource!((), BTreeMap<u64, ()>) will create a storage for the
/// () type, and the EntityId's will be of type u64.
#[macro_export] macro_rules! impl_resource {
    ($name:ty, $init:expr) => {
        impl genecs::static_storage::StaticStorage<$name> for $name {
            unsafe fn get_static() -> &'static mut ($name, (u64, u64)) {
                // Import std::sync::Once and std::sync::ONCE_INIT as the user might not import themselves
                use std::sync::{Once, ONCE_INIT};
                // Use an Inner type to call ::defualt as types with generic arguments will cause an error
                type Storage = $name;
                // Initialize the static storage to a const (Option, Once)
                static mut STATIC_STORAGE : (Option<($name, (u64, u64))>, Once) = (None, ONCE_INIT);
                // At runtime Change the static storages None to a Some($storage, (u64, u64))
                STATIC_STORAGE.1.call_once(|| {
                    STATIC_STORAGE.0 = Some(($init, (0,0)));
                });
                // Coerce a mutable refrence into a mutable pointer
                let ptr = &mut STATIC_STORAGE.0 as * mut Option<($name, (u64, u64))>;
                // Unwrap the option and return the inner storage
                match *ptr {
                    Some(ref mut x) => x,
                    None => unreachable!(),
                }
            }
        }
        impl Resource for $name {}
    }
}
