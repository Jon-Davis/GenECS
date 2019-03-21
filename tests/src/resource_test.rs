
#[test]
fn component_test(){
    use genecs::resource::Resource;

    struct SimpleTuple(u64);
    impl_resource!(SimpleTuple, SimpleTuple(0));

    let mut guard = acquire!(Write(SimpleTuple));
    let (res,) = guard.get();
    res.0 = 5;
    drop(guard);
    let mut guard = acquire!(Read(SimpleTuple));
    let (res,) = guard.get();
    assert!(res.0 == 5);
}