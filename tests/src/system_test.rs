

#[test]
fn system_test(){
    use genecs::system::System;

    static mut SYSTEM_TEST : usize = 0;

    enum Mather {
        Add(usize),
        Mul(usize),
        Div(usize),
    }

    impl System for Mather {
        fn run(&mut self) {
            // needed to modify static variable
            unsafe {
                match self {
                    Mather::Add(i) => SYSTEM_TEST += *i,
                    Mather::Mul(i) => SYSTEM_TEST *= *i,
                    Mather::Div(i) => SYSTEM_TEST /= *i,
                }
            }
        }
    }
    let mut add_one = Mather::Add(1);
    let mut mul_twe = Mather::Mul(12);
    let mut div_thr = Mather::Div(3);

    dispatch!(add_one, mul_twe, div_thr);
    unsafe {
        assert!(SYSTEM_TEST == 4);
    }
}

#[test]
fn system_par_test(){
    use genecs::system::System;
    use std::sync::Mutex;

    lazy_static! {
        static ref SYSTEM_TEST : Mutex<usize> = Mutex::new(0);
    }

    enum Mather {
        Add(usize),
    }

    impl System for Mather {
        fn run(&mut self) {
            match self {
                Mather::Add(i) => {
                    let val = SYSTEM_TEST.lock();
                    match val {
                        Ok(mut value) => *value += *i,
                        _ => panic!("Failed to get mutex lock, this might be a test error"),
                    }
                },
            }
        }
    }
    
    let mut add_zero = Mather::Add(0);
    let mut add_one = Mather::Add(1);
    let mut add_two = Mather::Add(2);
    let mut add_three = Mather::Add(3);

    dispatch!(add_zero);
    dispatch_parallel!(&mut add_one, &mut add_two, &mut add_three);
    assert!(*SYSTEM_TEST.lock().unwrap() == 6);
    dispatch!(add_one,add_two,add_three);
    assert!(*SYSTEM_TEST.lock().unwrap() == 12);
}