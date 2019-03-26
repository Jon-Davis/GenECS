pub trait System {
    /// The run function will be called and passed a mutable refrence to itself 
    /// whenever the system is dispatched
    fn run(&mut self);
}

/// the dispatch! macro is used 
#[macro_export] macro_rules! dispatch {
    ($($system:expr),*) => {
        $($system.run();)*
    };
}

#[macro_export] macro_rules! dispatch_parallel {
    ($($system:expr),*) => {
        genecs::crossbeam_utils::thread::scope(|s| {
            $(
                s.spawn(|_| {
                    $system.run();
                });
            )*
        }).unwrap();
    };
}