use amethyst::core::{SystemDesc, Transform};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadStorage, System, SystemData, World, WriteStorage};

#[derive(SystemDesc)]
pub struct DroneSystem;

impl<'s> System<'s> for DroneSystem {
    type SystemData = (WriteStorage<'s, Transform>, ReadStorage<'s, Drone>);

    fn run(&mut self, (mut transforms, drones): Self::SystemData) {
        for (drone, transform) in (&drones, &mut transforms).join() {
            // let movement
        }
    }
}
