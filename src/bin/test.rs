extern crate specs_test;
extern crate specs;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano;
use specs::*;
use specs_test::components::*;


fn main() {
    let mut world = World::new();
    println!("world created");
    world.register::<CompPosition>();
    println!("position comp registered");

    let new_entities = world
        .create_iter()
        .take(4)
        .collect::<Vec<_>>();

    for i in 0..new_entities.len() {
        world.write().pass().insert(new_entities[i], CompPosition(i as i32,i as i32));
    }
    println!("{} entity with position created",new_entities.len());

    let mut planner = Planner::<()>::new(world);
    planner.run0w1r(|pos: &CompPosition| {
        println!("Entity position: {:?}", pos);
    });


    planner.wait();

}