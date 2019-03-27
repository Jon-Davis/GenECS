# GenECS
A general purpose, parallel Entity Component System designed to be fast and avoid dynamic types and unnecessary heap allocations.

## Entitites
Entities serve a similar role to Structs. The difference is that Entities don't actually contain any fields. 
Instead an Entity is simply a number that Components associate with. This allows for Entities to be dyanmic at runtime, they
can add and remove Components during the execution of the program.

```rust
use genecs::entity::{Entity};

// to create a new entity simply call the new function
let entity = Entity::new();

// to add components to the entity, you need their associated storages
// but once you have them, you simply call the add function, note you can
// only have one instance of a component on a given entity
// see the component section to find out how to get a Component's Storage
entity.add(SomeComponentStorage, SomeComponent::new());

// to get the component of an entity, call get
// this function returns an Option<&SomeComponent>
// see the component section to find out how to get a Component's Storage
entity.get(SomeComponentStorage)

// to get a mutable refrence of a component to an entity,
// you need a mutable refrence to the components storage
// but when you have that you simply call get_mut()
// see the component section to find out how to get a Component's Storage
entity.get_mut(SomeComponentStorage)

// you can also delete components of an entity
// see the component section to find out how to get a Component's Storage
entity.rm(SomeComponentStorage);

// Entity is a wrapper around an EntityId which itself is just a usize number
// to get an Entity wrapper from an Id simply call the from methods
Entity::from(some_entity_id);
```

## Components
### Defining Components
Fields to Structs are Components to Entities. Like fields Components can be of any type, they simply need to Imlement the Component
trait. Since rust doesn't support associated statics, this can be a bit involved, luckily this crate provides a generic macro
for implementing the Component Trait.
```rust
use genecs::entity::EntityID;
use genecs::component::Component;

struct MyType();
impl_componet!(MyType, BTreeMap<EntityId, MyType>);
```
What this macro does is it takes in some type as it's first argument, in this case MyType, and a ComponentStorage<MyType>.
All instences of MyType will be stored in a static instance of this ComponentStorage. ComponentStorage is a rust Trait that simply requires the methods
add(entity_id, component), rm(entity_id), get(entity_id), and get_mut(entity_id). The standard BTreeMap implements Component Storage, but other data structures
can be defined as ComponentStorages by implementing the trait.

### Getting Components
As shown above, Entities need a component storage to add and remove components. so in order to do anything with them, you need
to be able to retrieve the ComponentStorages. You could do this directly, but it is unsafe. Instead GenECS provides a safe and thread
safe means of acquiring Components and that is the acquire! macro
```rust
// the acquire macro returns a guard, that will handle the releasing of the resources for you
let mut guard = acquire!(Read(CompA), Write(CompB), Read(CompC))

// to get the refrences to the ComponentStorages call the get method
let (a,b,c) = guard.get()

// The guard returns the ComponentStorages in their base type
// that means if CompA, CompB, and CompC are stored are BTreeMaps
// the get method returns (&BTreeMap, &mut BTreeMap, &BteeMap)
```
### Using Components
The most common way to use components is to use a join operation, where you get all the entities that contains paticular 
Components and iterate through them and apply some function. Unfortanetly GenEcs has no such function, Fortanetly I have these
functions available in another crate [kv_join](https://github.com/Jon-Davis/kv_join). An example of using components stored
in BTreeMaps and using the kv_join crate is below:
```rust
let mut guard = acquire!(Read(CompA), Write(CompB), Read(CompC))
let (a,b,c) = guard.get()

for (key, (a_value, b_value, c_value)) in kvand_join!(a.iter(), b.iter_mut(), c.iter()){
   // some process that likely modifies b using values a and c
}
```
## Resources
Resources are similar to Components, in fact they use almost all of the same code. The difference is, there is only 1 instance of a resource
and they do not have EntityIDs. An example of Resources would be a Texture that is refrenced by multiple Sprite Components in order to save on
memory. There is a slightly different macro for creating them.
```rust
struct MyType(usize)
impl_resource!(MyType, Mytype(5))
```
The macros first argument is the Type you want to make a Resource, the second argument is an Expression that will be run to initialize
the value.
## Systems
If Entities are like Structs and Components are like fields, then Systems are like Methods. Systems are simple structs that run a single
function and perform a task on Components. To create a system, create a struct with whatever data the system will need, then impl the System
trait which requires a single run function
```rust
Struct MySystem(/*System Data*/)
impl System for MySystem {
  fn run(&mut self) {
    // do something
  }
} 
```
Systems can be run in two ways: 
sequentially
```rust
let system_a = ..
let system_b = ..
let system_c = ..

dispatch!(&mut system_a, &mut system_b, &mut system_c)
```
parallel
```rust
let system_a = ..
let system_b = ..
let system_c = ..

dispatch_parallel!(&mut system_a, &mut system_b, &mut system_c)
```
you can control exactly when systems runs and stage them in any order you want
```rust
let input_system = ..
let time_system = ..
let physics_system = ..
let score_system = ..
let music_system = ..
let redner_system = ..

loop {
  dispatch!(&mut input_system, &mut time_system)
  dispatch_parallel!(&mut physics_system, &mut score_system, &mut music_system)
  dispatch!(&mut render_system)
}
```
