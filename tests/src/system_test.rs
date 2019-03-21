

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