use amethyst::{
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::Camera,
    ui::{UiCreator, UiFinder, UiText},
    utils::application_root_dir,
};

use serialport::{SerialPortSettings, SerialPortType};

// mod compass;

use crate::cobs_buffer::*;
// use crate::compass;
use crate::config::ReceiverSettings;
use crate::serial;
use crate::usbreader::Receiver;
// use crate::AppState;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Magnetometer {
    x: i16,
    y: i16,
    command: String,
}

#[derive(Default)]
pub struct App {
    ui_root: Option<Entity>,
    // Reference to the magnetometer data from the embedded device, which we want to modify
    heading: Option<Entity>,
    tx_connected: Option<Entity>,
}

fn watch_serial(config: &str) -> ! {
    let rx_settings = ReceiverSettings::load(&config).unwrap();

    let rx = Receiver::new((rx_settings.vid, rx_settings.pid)).unwrap();

    match rx.is_connected() {
        Ok(_r) => {
            let port_name = serial::serial_port((rx_settings.vid, rx_settings.pid)).unwrap();
            let mut settings: SerialPortSettings = Default::default();
            settings.baud_rate = rx_settings.baud_rate;

            match serialport::open_with_settings(&port_name, &settings) {
                Ok(mut port) => {
                    let mut serial_buf: Vec<u8> = vec![0; 256];
                    let mut deser_buf = Buffer::new();
                    println!("Receiving data on {}", &port_name);

                    loop {
                        if let Ok(t) = port.read(serial_buf.as_mut_slice()) {
                            let mut window = &serial_buf[..t];

                            'cobs: while !window.is_empty() {
                                use BufferResult::*;
                                window = match deser_buf.write::<Magnetometer>(&window) {
                                    Consumed => break 'cobs,
                                    Overfull(new_window) => new_window,
                                    DeserError(new_window) => new_window,
                                    Success { data, remaining } => {
                                        let theta = (data.y as f32).atan2(data.x as f32);
                                        // let degrees = compass::coordinates_to_degrees((
                                        // data.x as f32,
                                        // data.y as f32,
                                        // ));

                                        println!("theta: {:?}", theta);

                                        remaining
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
                    ::std::process::exit(1);
                }
            }
        } // Err(_) => println!("Please connect the device"),
    }
}

impl SimpleState for App {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.register::<Drone>();
        initialize_drone(world);
        initialize_camera(world);

        self.ui_root =
            Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/main.ron", ())));

        let app_root = application_root_dir().unwrap();
        let config_dir = app_root.join("config");
        let config_path = config_dir.join("config.ron");

        // match rx.listen() {
        // Ok(v) => v,
        // Err(_) => (),
        // };
        // world.insert(AppState {
        // assets_dir: assets_dir.clone(),
        // config_dir,
        // transceiver_cx: false,
        // });
        // let fetched = world.try_fetch::<AppState>();

        // if let Some(fetched_resource) = fetched {
        // // Dereference to access data
        // println!("RX Config path: {:?}", *fetched_resource);
        // } else {
        // println!("No AppState resource available")
        // }
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = state_data;

        if self.heading.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("heading") {
                    self.heading = Some(entity);
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

        if let Some(heading) = self.heading.and_then(|entity| ui_text.get_mut(entity)) {
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

        if let Some(tx_connected) = self.tx_connected.and_then(|entity| ui_text.get_mut(entity)) {
            tx_connected.text = String::from("not connected");
            // if let Ok(value) = heading.text.parse::<i32>() {
            // let mut new_value = value * 10;
            // if new_value > 100_000 {
            // new_value = 1;
            // }
            // heading.text = new_value.to_string();
            // } else {
            // heading.text = String::from("1");
            // }
        }
        Trans::None
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
