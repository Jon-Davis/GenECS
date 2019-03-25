pub trait System {
    /// The run function will be called and passed a mutable refrence to itself 
    /// whenever the system is dispatched
    fn run(&mut self);
}

/// the dispatch! macro is used 
#[macro_export] macro_rules! dispatch {
    ([$($system:tt),*]) => {
        // launch threads and recurse on all repitions, collect all threadIDs into an array handles
        let mut handles = [$(Some(std::thread::spawn(move || {dispatch!($system); })),)*];
        // join all the threads
        for handle in &mut handles {
            // since JoinHandles require ownership to join, and we cant take ownership out
            // of an array without creating undefined memory, we us an Option<JoinHandle> 
            // and the take method to gain ownership of the JoinHandles.
            let handle = handle.take();
            match handle {
                Some(handle) => handle.join().unwrap(),
                _ => unreachable!(),
            };
        }
    };
    (($($system:tt),*)) => {
        // recurse on repetition
        $(dispatch!($system);)*
    };
    ($system:tt) => {
        $system.run()
    };
    ($($system:tt),*) => {
        // recurse on repetition
        $(dispatch!($system);)*
    };
}