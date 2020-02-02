mod cobs_buffer;
mod compass;
mod config;
mod serial;
mod usbreader;

use crate::cobs_buffer::*;
use crate::config::ReceiverSettings;
use usbreader::Receiver;

use serde::{Deserialize, Serialize};
use serialport::{SerialPortSettings, SerialPortType};

use amethyst::{
    config::Config,
    ecs::World,
    prelude::*,
    renderer::{plugins::RenderToWindow, types::DefaultBackend, RenderingBundle},
    utils::application_root_dir,
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Magnetometer {
    x: i16,
    y: i16,
    command: String,
}

struct AppState {
    pub transceiver_cx: bool,
}

pub struct App;

impl SimpleState for App {}

fn main() -> amethyst::Result<()> {
    // Debug
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config_dir = app_root.join("config");
    let assets_dir = app_root.join("assets");

    let config_path = config_dir.join("config.ron");
    let display_path = config_dir.join("display.ron");

    let rx_settings = ReceiverSettings::load(&config_path)?;

    let mut world = World::empty();

    let app_state = AppState {
        transceiver_cx: false,
    };

    world.insert(app_state);

    let app_data = GameDataBuilder::default().with_bundle(
        RenderingBundle::<DefaultBackend>::new().with_plugin(
            RenderToWindow::from_config_path(display_path)?.with_clear([
                // Linear colorspace
                f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // R
                f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // G
                f32::powf((30.0 / 255.0 + 0.055) / 1.055, 2.4), // B
                1.0,                                            // A
            ]),
        ),
    )?;

    let mut app = Application::new(assets_dir, App, app_data)?;
    app.run();

    let fetched = world.try_fetch_mut::<AppState>();
    if let Some(mut fetched_resource) = fetched {
        assert_eq!(fetched_resource.transceiver_cx, false);
        fetched_resource.transceiver_cx = true;
        assert_eq!(fetched_resource.transceiver_cx, true);
    } else {
        println!("No AppState present in `World`");
    }

    let rx = Receiver::new((rx_settings.vid, rx_settings.pid))?;

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
                                        let degrees = compass::coordinates_to_degrees((
                                            data.x as f32,
                                            data.y as f32,
                                        ));

                                        println!("theta: {:?}, degrees: {:?}", theta, degrees);

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
        }
        Err(_) => println!("Please connect the device"),
    }

    match rx.listen() {
        Ok(v) => v,
        Err(_) => (),
    };

    Ok(())
}
