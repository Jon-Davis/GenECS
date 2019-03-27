
/*
commented out in favor of full_test as the tests share a common EntityID register

#[test]
fn entity_test(){
    use genecs::entity::EntityRegister;

    assert!(EntityRegister::get_new_id() == 0);
    assert!(EntityRegister::get_new_ids(5) == (1..6));
}*/