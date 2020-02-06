use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::Camera,
    ui::{UiCreator, UiFinder, UiText},
    // utils::application_root_dir,
    window::ScreenDimensions,
};

use std::io;
use std::sync::{mpsc, Arc, Mutex};

#[derive(Default)]
pub struct CompassUI {
    pub heading: Option<Entity>,
}

#[derive(Default)]
pub struct App {
    ui_root: Option<Entity>,
    compass_ui: CompassUI,
    tx_connected: Option<Entity>,
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

        // Load and assign "Heading" UI element
        if self.compass_ui.heading.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("heading") {
                    self.compass_ui.heading = Some(entity);
                }
            })
        }

        if self.tx_connected.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("tx_connected") {
                    self.tx_connected = Some(entity);
                }
            })
        }

        // if !self.paused {
        let mut ui_text = world.write_storage::<UiText>();

        if let Some(heading) = self
            .compass_ui
            .heading
            .and_then(|entity| ui_text.get_mut(entity))
        {
            if let Ok(value) = heading.text.parse::<i32>() {
                let mut new_value = value * 10;
                if new_value > 100_000 {
                    new_value = 1;
                }
                heading.text = new_value.to_string();
            } else {
                heading.text = String::from("1");
            }
        }

        // TODO: Implement USB transceiver connection status
        if let Some(tx_connected) = self.tx_connected.and_then(|entity| ui_text.get_mut(entity)) {
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
