use crate::static_storage::StaticStorage;

/// Resources are globally stored data where only 1 instance of
/// a type is stored. 
pub trait Resource : StaticStorage<Self> + 'static {}

/// impl_resource! macro implements the Resource Trait on a given struct
/// The first argument is the struct that Resource will be 
/// The second argument is an expression that will initialize the resource
/// 
/// For example impl_resource!(usize, 4 + 5) will initialize a global usize resource
/// with a defualt value of 9, calling acquire!(Read(usize)) will return the value of this resource
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
