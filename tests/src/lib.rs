#[allow(unused_imports)]
#[macro_use] extern crate genecs;
#[allow(unused_imports)]
#[macro_use] extern crate kv_join;
mod component_test;
mod resource_test;
mod system_test;
mod entity_test;

/*
fn system_test(){
    use genecs::system::System;

    struct SimpleTuple(u64);
    impl System for SimpleTuple {
        fn run(&mut self){
            println!("{}", self.0);
        }
    }

    let mut a = SimpleTuple(0);
    let mut b = SimpleTuple(1); 
    let mut c = SimpleTuple(2);
    let mut d = SimpleTuple(3); 
    let mut e = SimpleTuple(4); 
    let mut f = SimpleTuple(0);
    let mut g = SimpleTuple(1); 
    let mut h = SimpleTuple(2);
    let mut i = SimpleTuple(3); 
    let mut j = SimpleTuple(4); 
    let mut k = SimpleTuple(4); 
    let mut l = SimpleTuple(4); 


    dispatch!(a,[b,(c, d, e)],[f,g,(h,i,[j,k])],l);
    // a runs
    // a completes
    // b and c run, (in paralell)
    // d runs after c, (possibly in paralell to b (shares thread with c))
    // e runs after d, (possibly in paralell to b (shares thread with c))
    // b c d and e have all completed
    // f, g, and h run (possibly in paralell)
    // i runs after h (possibly in paralell with f and g (shares thread with h))
    // j and k run after i (in paralell with each other, and possibly in paralell with f and g)
    // f g h i j and k have all completed
    // l runs
}*/