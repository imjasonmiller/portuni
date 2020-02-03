use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage},
    prelude::*,
    renderer::Camera,
};

pub struct App;

impl SimpleState for App {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.register::<Drone>();
        initialize_drone(world);
        initialize_camera(world);
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 1.0);

    world
        .create_entity()
        .with(Camera::standard_3d(500.0, 500.0))
        .with(transform)
        .build();
}

pub struct Drone;

impl Drone {
    fn new() -> Drone {
        Drone {}
    }
}

impl Component for Drone {
    type Storage = DenseVecStorage<Self>;
}

fn initialize_drone(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 0.0);

    world
        .create_entity()
        .with(Drone::new())
        .with(transform)
        .build();
}
