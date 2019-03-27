#![allow(non_camel_case_types)]

#[test]
fn full_test(){
    use kv_join;
    use genecs::component::{Component};
    use genecs::entity::{EntityID, Entity};
    use genecs::system::System;
    use std::collections::BTreeMap;

    /// Create several components
    struct CompA(usize);
    struct CompB(f64);
    struct CompC(bool);
    struct CompD(f64);
    /// implment the Component trait on these structs
    impl_component!(CompA, BTreeMap<EntityID, CompA>);
    impl_component!(CompB, BTreeMap<EntityID, CompB>);
    impl_component!(CompC, BTreeMap<EntityID, CompC>);
    impl_component!(CompD, BTreeMap<EntityID, CompD>);

    /// Create severl systems
    struct InitializeEntities();
    struct Add_Usize_System(usize);
    struct Add_CompD_If_CompA_and_CompB();
    struct Multiply_CompD_If_CompC(f64);

    /// This system creates 4 entities
    impl System for InitializeEntities{
        fn run(&mut self) {
            // acquire the storages that will be used to create the new entities
            let mut guard = acquire!(Write(CompA), Write(CompB), Write(CompC));
            let (a,b,c) = guard.get();

            // create several entities
            Entity::new()
                .add(*a, CompA(1))
                .add(*c, CompC(true));
            Entity::new()
                .add(*a, CompA(10))
                .add(*b, CompB(0.5))
                .add(*c, CompC(true));
            Entity::new()
                .add(*a, CompA(10))
                .add(*b, CompB(0.5))
                .add(*c, CompC(false));
            Entity::new()
                .add(*b, CompB(100.0005))
                .add(*c, CompC(false));
        }
    }

    /// This sytem takes all entities with a CompA component
    /// and adds a value to it
    impl System for Add_Usize_System {
        fn run(&mut self) {
            let mut guard = acquire!(Write(CompA));
            let (a,) = guard.get();
            for (_id, value) in a.iter_mut() {
                value.0 += self.0;
            }
        }
    }

    /// This sytem takes all the entiities with a CompA and CompB
    /// Component and adds a new Component CompD which is the sum of the two
    impl System for Add_CompD_If_CompA_and_CompB {
        fn run(&mut self) {
            let mut guard = acquire!(Write(CompA), Write(CompB), Write(CompD));
            let (a,b,d) = guard.get();
            for (entity_id, (a_value, b_value)) in kvand_join!(a.iter_mut(), b.iter_mut()) {
                let d_value = a_value.0 as f64 + b_value.0 as f64;
                let entity = Entity::from(entity_id);
                entity.add(*d, CompD(d_value));
            }
        }
    }

    /// A system will take all the entities with a CompD and CompC
    /// Componnent and multiply the D component if the C component is true
    impl System for Multiply_CompD_If_CompC {
        fn run(&mut self){
            let mut guard = acquire!(Write(CompD), Read(CompC));
            let (d,c) = guard.get();
            for (_entity_id, (d_value, c_value)) in kvand_join!(d.iter_mut(), c.iter()){
                if c_value.0 {
                    d_value.0 *= self.0
                }
            }
        }
    }

    // Create the actual instance of the systems
    let mut sys1 = InitializeEntities();
    let mut sys2 = Add_Usize_System(10);
    let mut sys3 = Add_CompD_If_CompA_and_CompB();
    let mut sys4 = Multiply_CompD_If_CompC(2.0);

    // dispatch the systems
    dispatch!(sys1, sys2, sys3, sys4);

    // run tests to ensure systems executed successfully
    let mut guard = acquire!(Read(CompA), Read(CompB), Read(CompC), Read(CompD));
    let (a_s,b_s,c_s,d_s) = guard.get();
    // check the first entity
    let entity = Entity::from(0);
    let a = entity.get(*a_s).expect("A value was unexpectedly none");
    let b = entity.get(*b_s);
    let c = entity.get(*c_s).expect("C value was unexpectedly none");
    let d = entity.get(*d_s);
    assert!(a.0 == 11);
    assert!(b.is_none());
    assert!(c.0 == true);
    assert!(d.is_none());
    // check the second entity
    let entity = Entity::from(1);
    let a = entity.get(*a_s).expect("A value was unexpectedly none");
    let b = entity.get(*b_s).expect("B value was unexpectedly none");
    let c = entity.get(*c_s).expect("C value was unexpectedly none");
    let d = entity.get(*d_s).expect("D value was unexpectedly none");
    assert!(a.0 == 20);
    assert!(b.0 == 0.5);
    assert!(c.0 == true);
    assert!(d.0 == 41.0);
    // check the thrd entity
    let entity = Entity::from(2);
    let a = entity.get(*a_s).expect("A value was unexpectedly none");
    let b = entity.get(*b_s).expect("B value was unexpectedly none");
    let c = entity.get(*c_s).expect("C value was unexpectedly none");
    let d = entity.get(*d_s).expect("D value was unexpectedly none");
    assert!(a.0 == 20);
    assert!(b.0 == 0.5);
    assert!(c.0 == false);
    assert!(d.0 == 20.5);
    // check the fourth entity
    let entity = Entity::from(3);
    let a = entity.get(*a_s);
    let b = entity.get(*b_s).expect("B value was unexpectedly none");
    let c = entity.get(*c_s).expect("C value was unexpectedly none");
    let d = entity.get(*d_s);
    assert!(a.is_none());
    assert!(b.0 == 100.0005);
    assert!(c.0 == false);
    assert!(d.is_none());
}