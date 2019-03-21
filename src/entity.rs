use std::sync::atomic::{AtomicUsize, Ordering};
pub struct EntityRegister {}

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