use std::sync::{Mutex, Condvar, Once, ONCE_INIT};

/// The StaticStorage trait is used as a building block for other types in the crate
/// StaticStorage contains a static of type Storage and also keeps track of the number
/// of readers and writers to it's contents. It also contains a mutex that can be used to
/// guarentee mutual exclusion to it's contents, however this needs to be manually handled by the
/// user.
pub trait StaticStorage<Storage : 'static> : Sized {
    /// read is an unsafe function that shouldn't be called directly. 
    /// This function retrieves the storage, and returns a refrence of it if there are currently no writers.
    /// An Error::IllegalReadGuardAcquire is returned if there is currently a writer in the storage.
    /// An Error::UnitializedStorage is returned if the storage hasn't been initialized.
    /// This function is NOT thread safe.
    /// This function does not block.
    /// 
    /// This function is used by the acquire! macro.
    unsafe fn read() -> Result<&'static Storage,Error> {
        let store = Self::get_static();
        match store {
            // So long as there are no writers, you can create a reader
            (storage,(i,0)) => {
                *i += 1; // Increment the number of readers
                return Ok(storage);
            },
            _ => Err(Error::IllegalReadGuardAcquire),
        }
    }

    /// write is an unsafe function that shouldn't be called directly. 
    /// This function retrieves the storage, and returns a mutable refrence of it if there are currently no readers or writers.
    /// An Error::IllegalWriteGuardAcquire is returned if there is currently a writer or reader in the storage.
    /// An Error::UnitializedStorage is returned if the storage hasn't been initialized.
    /// This function is NOT thread safe.
    /// This function does not block.
    /// 
    /// This function is used by the acquire! macro.
    unsafe fn write() -> Result<&'static mut Storage,Error> {
        let mut store = Self::get_static();
        match store {
            // So long as there are no readers or writers, you can create a writer
            (storage,(0,0)) => {
                (store.1).1 = 1; // Set the number of writers to 1
                return Ok(storage);
            },
            _ => Err(Error::IllegalWriteGuardAcquire),
        }
    }
    
    /// release_read_guard is an unsafe function that shouldn't be called directly.
    /// This function simply decrements the number of readers being tracked by the storage.
    /// This function does not actually guarentee that the read_guard is actually freed.
    /// An Error::AliasingDetected is returned if there is a reader and a writer, or multiple writers.
    /// An Error::UnacountedGuard is returned if the storage didn't have readers to free.
    /// This function is NOT thread safe.
    /// This function does not block.
    /// 
    /// This function is used in the desturctor of a Guard that is generated by the acquire! macro.
    unsafe fn release_read_guard() -> Result<(), Error>{
        let store = Self::get_static();
        match store {
            // If the number of readers is 0, then releasing a reader is an UnacountedGuard Error
            (_, (0, 0)) => return Err(Error::UnacountedGuard),
            // If the number of readers is greater than 0 and there are 0 writers then decrement number of readers
            (_, (r, 0)) => *r -= 1,
            // If there was a writer, then aliasing has occured, return an AliasingDetected error
            _ => return Err(Error::AliasingDetected),
        }
        Ok(())
    }

    /// release_write_guard is an unsafe function that shouldn't be called directly.
    /// This function simply decrements the number of writers being tracked by the storage.
    /// This function does not actually guarentee that the write_guard is actually freed.
    /// An Error::AliasingDetected is returned if there is a reader and a writer, or multiple writers.
    /// An Error::UnacountedGuard is returned if the storage didn't have a writer to free.
    /// This function is NOT thread safe.
    /// This function does not block.
    /// 
    /// This function is used in the desturctor of a Guard that is generated by the acquire! macro.
    unsafe fn release_write_guard() -> Result<(), Error> {
        let store = Self::get_static();
        match store {
            // If there is 1 writer and 0 readers then decrement the number of writers
            (_, (0, w)) if *w == 1 => *w -= 1,
            // If there is 0 writers than releasing a writer is an UnacountedGuard error
            (_, (0, 0)) => return Err(Error::UnacountedGuard),
            // If there are readers or multiple writers then return an AliasingDetected error
            _ => return Err(Error::AliasingDetected),
        }
        Ok(())
    }
    
    /// get_access is an unsafe function that returns the number of readers and the number of writers that
    /// are currently acessing the Storage.
    /// This function is NOT thread safe.
    /// This function does not block.
    /// 
    /// This function is used by the acquire! macro
    unsafe fn get_access() -> RWInfo { Self::get_static().1 }

    /// Since rust does not support associated statics (yet) this function needs to be implemented by the user. This function points to the static
    /// Storage. users can use the impl_component! or impl_resource! macro which will create the static Storage and implement this function. This function
    /// is unsafe because getting a mutable refrence to a static variable is unsafe.
    unsafe fn get_static() -> &'static mut (Storage, RWInfo);

    /// The MUTEX contains both a mutex and a condition variable, there is one of these per program
    /// and it is used to ensure mutual exclusion when accessing the Storages
    fn get_mutex() -> &'static (Mutex<()>,Condvar) {
        unsafe {
            // create the static variable, but initialize it to None, because Mutex and Condvar can't be const created
            static mut MUTEX : (Option<(Mutex<()>,Condvar)>, Once) = (None, ONCE_INIT);
            // Using the sync::Once initialize the Mutex and the Condvar to ensure this only happens once
            MUTEX.1.call_once(|| {
                MUTEX.0 = Some((Mutex::new(()), Condvar::new()));
            });
            // Coerce a mutable refrence into a mutable pointer
            let ptr = &mut MUTEX.0 as * mut Option<(Mutex<()>,Condvar)>;
            // Match the ptr to unwrap the option, then return the inner value
            match *ptr {
                Some(ref x) => x,
                None => unreachable!(),
            }
        }
    }
}

/// RWInfo contains the number of readers and the number of writers currently accessing the Storage
type RWInfo = (u64, u64);

/// Guard takes a tuple of refrences to StaticStorages T
/// and a desturctor F that frees all the resources in the tuple
/// T. This should be created using the acquire! macro
pub struct Guard<T, F : FnMut()>{
    storages: T,
    destructor: F,
}


impl<T, F : FnMut()> Guard<T, F>{
    /// Creates a new Guard
    pub fn new(t : T, f : F) -> Self {
        Self {
            storages : t,
            destructor : f
        }
    }
    /// gets the underlining resources protected by the guard
    pub fn get(&mut self) -> &mut T {
        &mut self.storages
    }
}

/// Call the destructor function on drop to release all the resources
/// aquired by the guard at creation
impl<T, F : FnMut()> Drop for Guard<T, F> {
    fn drop(&mut self) {
        (self.destructor)();
    }
}

/// An Enum that represents all the different types of errors
/// that can be generated by Static Storage
pub enum Error{
    AliasingDetected,           // Occurs when readers and writers are loaned at the same time
    UnacountedGuard,            // Occurs when you try to free a guard, but the rw_info says the guard doesn't exist
    IllegalReadGuardAcquire,    // Occurs when you try to gain read access with a writer already loaned
    IllegalWriteGuardAcquire,   // Occurs when you try to gain write access with readers or writers already loaned
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::AliasingDetected => write!(f, "Both read and write refrences to a storage were detected"),
            Error::IllegalReadGuardAcquire => write!(f, "Attempted to acquire a read guard for a static storage when a write guard exists"),
            Error::IllegalWriteGuardAcquire => write!(f, "Attempted to acquire a write guard for a static storage when a read guard or a write guard exists"),
            Error::UnacountedGuard => write!(f, "Attempted to release a guard that the static storage storage was unaware of."),
        }
    }
}

/// can_aquire should not be used directly as it is unsafe, but it checks whether or not a stroage is available to a reader or writer
#[macro_export] macro_rules! can_acquire {
    // In order for a mutable refrence to be valid, there can be no other mutable or immutable refrences out
    (Write($arg:tt)) => {
        match $arg::get_access() {
            (0,0) => true,
            _ => false,
        }
    };
    // In order for a refrence to be valid, there can be no other mutable refrences out
    (Read($arg:tt)) => {
        match $arg::get_access() {
            (_,0) => true,
            _ => false,
        }
    };
}

/// acquire_storage should not be used directly as it is unsafe, it simply calls write_storage or read_storage depending on whether Read(Struct) or Write(Struct) was passed
#[macro_export] macro_rules! acquire_storage {
    // If the Access was write then we want to acquire a mutable refrence
    (Write($arg:tt)) => {
        $arg::write().unwrap()
    };
    // If the Access was read then we want to acquire a refrence
    (Read($arg:tt)) => {
        $arg::read().unwrap() 
    };
}

/// release_storage should not be used directly as it is unsafe and is used the the construction of the destructor of the Component Guard struct
#[macro_export] macro_rules! release_storage {
    // If the Access was write then we want to release a write guard
    (Write($arg:tt)) => {
        $arg::release_write_guard().unwrap() 
    };
    // If the Access was read then we want to release a read guard
    (Read($arg:tt)) => {
        $arg::release_read_guard().unwrap()
    };
}

/// get_first retrieves the first type in a repetition
#[macro_export] macro_rules! get_first {
    ($type:tt, $(other:tt),*) => {$type};
}

/// The acquire! macro is used to retrieve an arbitrary number of StaticStorages
/// in a thread safe manner. The macro will only return once it can acquire all of the
/// StaticStorages with the requested permisions.
/// 
/// For example acquire!(Read(Type1), Write(Type2), Read(Type3)) will return a
/// ComponentGuard with refrences to the StaticStorages of Type1 and Type3, and
/// a mutable refrence to the ComponentStorage of Type2. 
/// 
/// This macro CAN block
/// This macro panics if an error occurs while acquiring or releasing resources
#[macro_export] macro_rules! acquire {
    // capture an arbitrary number of arguments of the form Access(Type)
    ($($access:tt($type:tt)),*) => {
        unsafe {
            use genecs::static_storage::{StaticStorage, Guard};
            // get the type of the first type so that we can call ::get_mutex()
            type first_type = get_first!($($type,)*);
            // retrieve and lock the static storage mutex
            let mut mutex = first_type::get_mutex().0.lock().expect("Failed to access Component mutex");
            // acquire can only return if it can acquire all of the requested items 
            let mut available = false;
            while available == false {
                // set available to true, so that it has an inital value
                available = true;
                // Check to see if all of the resources are available, if all are true then the and will
                // result in a true, if atleast one is false then the and is false.
                ($(available = available & can_acquire!($access($type)),)*);
                // if not all types were available go to sleep on the Conditional Variable
                if !available {
                    mutex = first_type::get_mutex().1.wait(mutex).unwrap();
                }
            }
            // Collect all the resources and store them in a tuple
            let storages = ($(acquire_storage!($access($type)),)*);
            // In order to maintain rust's idomatic vibe of no manual destruction for the user,
            // we create a lambda destructor that will be called by the created guard for freeing
            // all the aquired types
            let destructor = || {
                // acquire the mutex as we will be writing to the RW_Storages
                let _mutex = first_type::get_mutex().0.lock().expect("Failed to access Component mutex");
                // Undo the reads and the writes of the acquire
                ($(release_storage!($access($type)),)*);
                // wake up any threads that might be sleeping waiting for resources
                first_type::get_mutex().1.notify_all();            
            };
            // create and return a Component Guard 
            Guard::new(storages, destructor)
        }
    }
}