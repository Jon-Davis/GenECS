use core::any::TypeId;

/*
   fn my_system(config : SystemConfig) -> Result<()> {
	// configure the system, giving it the name "My System"
	// and access to the resources CompA, CompB, and CompC
	let system_data = config_system!(config, "my_system", (R(CompA), R(CompB), W(CompC)));
	
	//unpack the system resources into their own variables for convience
	let (ca, cb, cc) = system_data.resources;
	let bump_allocator = system_data.allocator;

	// execute the system
	.
	.
	.
}

fn my_system(config : SystemConfig) -> Result<()> {
	let system_data = {
		if config.used_token == true {
			return Result::ErrorUsedConfigToken
		}
		config.used_token = true;
		match config {
			SystemConfig::Query => {
				const RESOURCE_IDS : [TypeId; 3] = [typeid_of<CompA>, typeid_of<CompB>, typeid_of<CompC>];
				const NAME : str = "My System";
				return Result::Query(&RESOURCE_IDS, &NAME);
			}
			SystemConfig::Run => {
				(CompA::read(), CompB::read(), CompC::write())
			}
		}
	}
}
*/

pub enum Config {
	Run,
	Query,
} 

pub enum Result {
	Ok,
	Query(&'static (&'static [TypeId], &'static [bool], &'static str)),
}

pub struct SystemData {

}

impl SystemData {
	fn new() -> Self {
		SystemData {

		}
	}
}

#[macro_export] macro_rules! config_system {
	// @count_resources takes in the number of resources and counts them recursively by poping off the head, adding 1
    // then adding the recursion down to 0
	(@count_resources ($head:ty, $($tail:tt,)*)) => { 
        1 + config_system!(@count_resources ($($tail,)*)) 
    };
    (@count_resources ()) => {0};

	// @access_to_permisions is used when creating an array of permisions and converts Read/Write tokens into an Enum
    (@access_to_permisions (Write)) => { true };
    (@access_to_permisions (Read)) => { false };

	// The main body of the Macro
	( $config:ident, $name:expr, ($( $access:tt( $resource:ty ) ),*) ) => {
		match $config {
			genecs::system::Config::Query => {
				const SIZE : usize = config_system!(@count_resources ($($resource,)*));
				const RESOURCE_IDS : [core::any::TypeId; SIZE] = [$(core::any::TypeId::of::<$resource>(),)*];
				const RESOURCE_MUT : [bool; SIZE] = [$(config_system!(@access_to_permisions ($access)),)*];
				const NAME : str = $name;
				const META : (&[core::any::TypeId; SIZE], &[bool; SIZE], &str) = (&RESOURCE_IDS, &RESOURCE_MUT, &NAME);
				return genecs::system::Result::Query(&META);
			},
			genecs::system::Config::Run => {
				genecs::system::SystemData::new(/*TODO*/);
			}
		}
	}
}