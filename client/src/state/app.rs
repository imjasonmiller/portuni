use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::Camera,
    ui::{UiCreator, UiFinder, UiText},
    window::ScreenDimensions,
};

#[derive(Default, Debug)]
pub struct CompassUI {
    pub heading: Option<Entity>,
}

#[derive(Debug, Default)]
pub struct App {
    ui_root: Option<Entity>,
    pub compass_ui: CompassUI,
    trx_status: Option<Entity>,
}

impl SimpleState for App {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.register::<Drone>();
        initialize_drone(world);
        initialize_camera(world);

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/main.ron", ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = state_data;

        // Assign UI elements
        if self.compass_ui.heading.is_none() || self.trx_status.is_none() {
            world.exec(|finder: UiFinder| {
                self.compass_ui.heading = finder.find("heading");
                self.trx_status = finder.find("trx_status");
            })
        }

        // if !self.paused {
        let mut ui_text = world.write_storage::<UiText>();

        // TODO: Implement USB transceiver connection status
        if let Some(tx_connected) = self.trx_status.and_then(|entity| ui_text.get_mut(entity)) {
            tx_connected.text = String::from("not connected");
        }

        Trans::None
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 1.0);

    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    world
        .create_entity()
        .with(transform)
        .with(Camera::standard_3d(width, height))
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
