use std::sync::atomic::{AtomicUsize, Ordering};
use crate::component::{Component, ComponentStorage};
/// The EntityRegister is a empty struct that can be used to generate
/// new unique entity ids. The entity ids are of type usize and start
/// at 0. The Entity Register is effectively an atomic counter.
pub struct EntityRegister();
pub type EntityID = usize;

/// Entity is a simple wrapper around EntityID and interfaces with
/// the component system to allow for reduced boiler plate
pub struct Entity(EntityID);

impl<'a> Entity{

    /// Creates a new Entity with a new EntityID
    pub fn new() -> Self {
        Entity(EntityRegister::get_new_id())
    }

    /// Adds a component to this entity 
    pub fn add<C,S>(self, storage : &'a mut S, comp : C) -> Self where C : Component<S>, S : ComponentStorage<C, EntityID=usize> + 'static {
        storage.component_insert(self.0, comp);
        self
    }

    /// Removes a component from this entity
    pub fn rm<C,S>(self, storage : &'a mut S) where C : Component<S>, S : ComponentStorage<C, EntityID=usize> + 'static {
        storage.component_remove(&self.0);
    }

    /// Gets a refrence to a component of this entity
    pub fn get<C,S>(&self, storage : &'a S) -> Option<&'a C> where C : Component<S>, S : ComponentStorage<C, EntityID=usize> + 'static {
        storage.component_get(self.0)
    }

    /// Get a mutable refrence to a component of this entity
    pub fn get_mut<C,S>(&mut self, storage : &'a mut S) -> Option<&'a mut C> where C : Component<S>, S : ComponentStorage<C, EntityID=usize> + 'static {
        storage.component_get_mut(self.0)
    }

    /// Get the id of this entity
    pub fn get_id(&self) -> usize {
        self.0
    }
}

impl From<usize> for Entity{
    fn from(id : usize) -> Entity {
        Entity(id)
    }
}

impl From<&usize> for Entity{
    fn from(id : &usize) -> Entity {
        Entity(*id)
    }
}

/// Initialize EntityIDs to 0
static ENTITY_REGISTER : AtomicUsize = AtomicUsize::new(0);

impl EntityRegister {
    /// Returns a new unique id that can be used for this entity
    pub fn get_new_id() -> usize {
        ENTITY_REGISTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Returns a range of new unique ids that can be used for as entities
    pub fn get_new_ids(num: usize) -> std::ops::Range<usize> {
        let start = ENTITY_REGISTER.fetch_add(num, Ordering::Relaxed);
        (start..start+num)
    }
}