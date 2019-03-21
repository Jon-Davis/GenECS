/// Test the basics of the component functionality including acquiring, modifiying,
/// and joining
#[test]
fn component_test(){
    use kv_join;
    use genecs::component::{Component, ComponentStorage};
    use std::collections::BTreeMap;

    impl_component!(SimpleTuple, BTreeMap<u64, SimpleTuple>);
    struct SimpleTuple(u64);

    impl_component!(SimpleTuple2, BTreeMap<u64, SimpleTuple2>);
    struct SimpleTuple2(u64);

    impl_component!(SimpleTuple3, BTreeMap<u64, SimpleTuple3>);
    struct SimpleTuple3(u64);

    let mut components = acquire!(Write(SimpleTuple), Write(SimpleTuple2), Write(SimpleTuple3));
    let (st, st2, st3) = components.get();
    st.component_insert(3, SimpleTuple(1));
    st.component_insert(6, SimpleTuple(2));
    st.component_insert(9, SimpleTuple(3));
    st.component_insert(12, SimpleTuple(4));
    st.component_insert(15, SimpleTuple(5));

    st2.component_insert(0, SimpleTuple2(1));
    st2.component_insert(6, SimpleTuple2(2));
    st2.component_insert(12, SimpleTuple2(3));
    st2.component_insert(18, SimpleTuple2(4));
    st2.component_insert(24, SimpleTuple2(5));

    st3.component_insert(2, SimpleTuple3(1));
    st3.component_insert(4, SimpleTuple3(2));
    st3.component_insert(6, SimpleTuple3(3));
    st3.component_insert(8, SimpleTuple3(4));
    st3.component_insert(10, SimpleTuple3(5));
    st3.component_insert(12, SimpleTuple3(6));
    
    let mut iter = kva_join!(st.iter(), st2.iter(), st3.iter());
    let (k, (v1,v2,v3)) = iter.next().unwrap();
    assert!(*k == 6, "key of first join was incorrect");
    assert!(v1.0 == 2, "First value of first join was incorrect");
    assert!(v2.0 == 2, "Second value of first join was incorrect");
    assert!(v3.0 == 3, "Third value of first join was incorrect");
    let (k, (v1,v2,v3)) = iter.next().unwrap();
    assert!(*k == 12, "key of second join was incorrect");
    assert!(v1.0 == 4, "First value of second join was incorrect");
    assert!(v2.0 == 3, "Second value of second join was incorrect");
    assert!(v3.0 == 6, "Third value of second join was incorrect");
    assert!(iter.next().is_none());
} 

#[test]
fn write_then_read(){
    use genecs::component::{Component, ComponentStorage};
    use std::collections::BTreeMap;

    impl_component!(SimpleTuple, BTreeMap<u64, SimpleTuple>);
    struct SimpleTuple(u64);
    {
        let mut components = acquire!(Write(SimpleTuple));
        let (st,) = components.get();
        st.component_insert(3, SimpleTuple(1));
        st.component_insert(6, SimpleTuple(2));
        st.component_insert(9, SimpleTuple(3));
        st.component_insert(12, SimpleTuple(4));
        st.component_insert(15, SimpleTuple(5));
    }
    {
        let mut components = acquire!(Read(SimpleTuple));
        let (st,) = components.get();
        let mut iter = st.iter();
        let (k,v) = iter.next().unwrap();
        assert!(*k == 3 && v.0 == 1);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 6 && v.0 == 2);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 9 && v.0 == 3);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 12 && v.0 == 4);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 15 && v.0 == 5);
        assert!(iter.next().is_none());
    }
}

#[test]
fn multiple_reads(){
    use genecs::component::{Component};
    use std::collections::BTreeMap;

    impl_component!(SimpleTuple, BTreeMap<u64, SimpleTuple>);
    struct SimpleTuple(u64);
    let _components = acquire!(Read(SimpleTuple), Read(SimpleTuple), Read(SimpleTuple));
} 

#[test]
fn multiple_writes(){
    use genecs::component::{Component, ComponentStorage};
    use std::collections::BTreeMap;
    use std::{thread};
    use std::time::{Duration,Instant};

    impl_component!(SimpleTuple, BTreeMap<u64, SimpleTuple>);
    struct SimpleTuple(u64);
    let mut components = acquire!(Write(SimpleTuple));
    let (st,) = components.get();
    let now = Instant::now();
    let now2 = now.clone();
    let child = thread::spawn(move || {
        assert!(now2.elapsed().as_millis() < 100);
        let mut components = acquire!(Write(SimpleTuple));
        assert!(now2.elapsed().as_millis() >= 100);
        let (st,) = components.get();
        let mut iter = st.iter();
        let (k,v) = iter.next().unwrap();
        assert!(*k == 1 && v.0 == 1);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 2 && v.0 == 2);
        let (k,v) = iter.next().unwrap();
        assert!(*k == 3 && v.0 == 3);
        assert!(iter.next().is_none());
        st.component_remove(&2);
    });
    thread::sleep(Duration::from_millis(100));
    st.component_insert(1,SimpleTuple(1));
    st.component_insert(2,SimpleTuple(2));
    st.component_insert(3,SimpleTuple(3));
    drop(components);
    assert!(child.join().is_ok());
    let mut components = acquire!(Read(SimpleTuple));
    let (st,) = components.get();
    let mut iter = st.iter();
    let (k,v) = iter.next().unwrap();
    assert!(*k == 1 && v.0 == 1);
    let (k,v) = iter.next().unwrap();
    assert!(*k == 3 && v.0 == 3);
    assert!(iter.next().is_none());
}