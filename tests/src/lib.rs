#[allow(unused_imports)]
#[macro_use] extern crate genecs;
#[allow(unused_imports)]
#[macro_use] extern crate kv_join;
#[allow(unused_imports)]
#[macro_use] extern crate lazy_static;
/*
mod component_test;
mod resource_test;
mod system_test;
mod entity_test;
mod full_tests;*/

use genecs::system;

struct CompA();
struct CompB();
struct CompC();

fn my_system(config : system::Config) -> system::Result {
	// configure the system, giving it the name "My System"
	// and access to the resources CompA, CompB, and CompC
	let system_data = config_system!(config, "my_system", (Read(CompA), Read(CompB), Write(CompC)));
	
}