pub trait System {
    /// The run function will be called and passed a mutable refrence to itself 
    /// whenever the system is dispatched
    fn run(&mut self);
}

/// the dispatch! simply runs all the systems given sequentially
#[macro_export] macro_rules! dispatch {
    ($($system:expr),*) => {
        $($system.run();)*
    };
}

/// the dispatch_parallel! macro spawns a thread for each system, then runs them
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