

#[test]
fn entity_test(){
    use genecs::entity::EntityRegister;

    assert!(EntityRegister::get_new_id() == 0);
    assert!(EntityRegister::get_new_ids(5) == (1..6));
}