use core::any::TypeId;
use core::cell::RefCell;

pub trait System {
    type Resources;

    /// The run function will be called and passed a mutable refrence to itself 
    /// whenever the system is dispatched
    fn run(&mut self, res : &mut Self::Resources);

    fn get_type_ids() -> &'static [TypeId];
    fn get_type_permisions() -> &'static [Permisions];
    unsafe fn acquire_resources() -> Self::Resources;
}

/// SystemInterface is an abstraction over the System type that allows for systems to
/// be turned into trait objects, and still run.
pub trait SystemInterface {
    unsafe fn start(&mut self);
}

/// SystemReference is an enum that is used to abstract what kind of refrences the dispatcher
/// can use. it supports mutable refrences but also RefCells which are useful if multiple dispatchers
/// want to use the same system
pub enum SystemReference<'a,S,R> where S : System<Resources=R> {
    MutRef(&'a mut S),
    RefCell(&'a RefCell<S>),
}

impl<'a,S,R> SystemInterface for SystemReference<'a,S,R> where S : System<Resources=R>  {
    unsafe fn start(&mut self) {
        match self {
            // if the SystemReference is a mutable refrence, simply acquire the resources
            // run the system.
            SystemReference::MutRef(mut_ref) => {
                let mut resources = S::acquire_resources();
                mut_ref.run(&mut resources);
            }

            // if the SystemReference is a Refrence to a RefCell of the system, the borrow
            // mutable access to the system, acquire the resources, then run the system
            SystemReference::RefCell(ref_cell) => {
                let mut mut_ref = ref_cell. borrow_mut();
                let mut resources = S::acquire_resources();
                mut_ref.run(&mut resources);
            },
        }
    }
}
/// Allow RefCells of systems to be converted into SystemReference enum
impl<'a,S,R> From<&'a RefCell<S>> for SystemReference<'a,S,R> where S : System<Resources=R> {
    fn from(f : &'a RefCell<S>) -> Self {
        SystemReference::RefCell(f)
    }
}

/// Allow mutable refrences to systems to be converted into SystemReference enum
impl<'a,S,R> From<&'a mut S> for SystemReference<'a,S,R> where S : System<Resources=R> {
    fn from(f : &'a mut S) -> Self {
        SystemReference::MutRef(f)
    }
}

#[macro_export] macro_rules! system_resources {
    // @count_resources takes in the number of resources and counts them recursively by poping off the head, adding 1
    // then adding the recursion down to 0
    (@count_resources $head:ty, $($tail:tt,)*) => { 
        1 + system_resources!(@count_resources $($tail,)*) 
    };
    (@count_resources) => {0};

    // @access_to_refrences converts the Write(Resource) and Read(Resource) in refrences such as &mut Resource or &Resource
    (@access_to_refrences Write($arg:ty)) => { &mut $arg };
    (@access_to_refrences Read($arg:ty)) => { &$arg };

    // @acquire_resource calls the underlining function to read or write to the resource, this returns a &Resource or &mut Resource
    (@acquire_resource Write($arg:ty)) => {{ 
        type Resource = $arg; 
        Resource::write() 
    }};
    (@acquire_resource Read($arg:ty)) => {{ 
        type Resource = $arg; 
        Resource::read()
    }};

    // @access_to_permisions is used when creating an array of permisions and converts Read/Write tokens into an Enum
    (@access_to_permisions Write) => { genecs::system::Permisions::Write };
    (@access_to_permisions Read) => { genecs::system::Permisions::Read };

    // The core of the system_resources macro, this implements the associated type, and unsafe functions of a system automatically
    // so a user only ever needs to implement the run() method
    ($($access:tt($type:ty)),* $(,)*) => {
        // The Resources type is set to a tuple of mutable and immutable refrences
        type Resources = ( $( system_resources!(@access_to_refrences $access($type)) ,)* );

        // the get_type_ids function defines a constant array containg all the typeids of the associated resources and returns a refrence to it
        fn get_type_ids() -> &'static [core::any::TypeId] {
            const SIZE : usize = system_resources!(@count_resources $($type,)*);
            const IDS : [core::any::TypeId; SIZE] = [$(core::any::TypeId::of::<$type>(),)*];
            &IDS
        }

        // get_type_permisions function defines a constant array containg the R/W permisions of each resource for use in conflicts
        fn get_type_permisions() -> &'static [genecs::system::Permisions] {
            const SIZE : usize = system_resources!(@count_resources $($type,)*);
            const PERMISIONS : [genecs::system::Permisions; SIZE] = [$(system_resources!(@access_to_permisions $access),)*];
            &PERMISIONS
        }

        // acuqire_resources function calls the get or read function for every resource and returns a tuple of type Resources
        unsafe fn acquire_resources() -> Self::Resources {
            ( $( system_resources!(@acquire_resource $access($type)) ,)* )
        }
    }
}

pub fn get_longest_dependency_chain(input : &mut [(usize, usize, bool)], relationships : &[&[usize]]){
    loop {
        // find a system to evaluate
        let (mut system, mut visited) = (0, true);
        for sys in input.iter_mut() {
            if sys.2 == true {
                continue
            }
            system = sys.0;
            visited = sys.2;
            break;
        }

        // if there are no more systems to evaluate, return
        if visited {
            return;
        }

        visit(system, input, relationships);
    }
}

fn visit(visiting : usize, input : &mut [(usize, usize, bool)], relationships : &[&[usize]]) -> usize {
    // if this node has already been visited, return
    if input[visiting].2 == true {
        return 0;
    }
    input[visiting].2 = true;
    let mut max_chain = 0;
    // vist every one of the dependencies
    for dependency in relationships[visiting]{
        let chain = 1 + visit(*dependency, input, relationships);
        if chain > max_chain {
            max_chain = chain;
        }
    }
    input[visiting] = (visiting, max_chain, true);
    return max_chain;
}


#[macro_export] macro_rules! build_dispatcher {
    // sets the $name variable equal to a tuple of ($value, $depencies) then recurses on itself incrementing the value by 1
    (@set_dependencies $value:expr, $name:ident, $dependencies:expr, $($tail:tt),*) => {
        let $name = ($value, $dependencies);
        build_dispatcher!(@set_dependencies $value + 1, $($tail),*)
    };
    (@set_dependencies $value:expr, $name:ident, $dependencies:expr) => {
        let $name = ($value, $dependencies);
    };

    // The main pattern for build_dispatcher! method that sorts the systems by dependencies, then stages the systems to avoid conflict
    // the returns a dispatcher that can be run
    ( $( ($rf:expr, $name:ident, [$($dependencies:expr),* $(,)*]) ),* $(,)*) => {
        {
            build_dispatcher!(@set_dependencies 0, $($name,[$($dependencies.0,)*]),*);
            let relationships = [$(&$name.1,)*];
        }
    };
}

pub enum Permisions {
    Read,
    Write,
}