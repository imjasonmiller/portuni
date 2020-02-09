use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use amethyst::{
    config::Config,
    core::{SystemDesc, Transform},
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, SystemData, Write, WriteStorage},
    prelude::*,
    ui::{UiFinder, UiText},
    utils::application_root_dir,
    utils::tag::Tag,
};

use serialport::{open_with_settings, SerialPortSettings};

use crate::config::TransceiverSettings;
use crate::transceiver::TransceiverDevice;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Telemetry {
    mag_x: i16,
    mag_y: i16,
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
    temp: i8,
}

use crate::utils::interp::MovingAverage;

pub struct TransceiverCodecSystem {
    trx_recv: Option<Arc<Mutex<Receiver<Telemetry>>>>,
    mag_x_avg: MovingAverage,
    mag_y_avg: MovingAverage,
    gyro_x_avg: MovingAverage,
    gyro_y_avg: MovingAverage,
    gyro_z_avg: MovingAverage,
}

impl TransceiverCodecSystem {
    pub fn new() -> TransceiverCodecSystem {
        TransceiverCodecSystem {
            trx_recv: None,
            mag_x_avg: MovingAverage::new(32, None),
            mag_y_avg: MovingAverage::new(32, None),
            gyro_x_avg: MovingAverage::new(16, None),
            gyro_y_avg: MovingAverage::new(16, None),
            gyro_z_avg: MovingAverage::new(16, None),
        }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, TransceiverCodecSystem> for TransceiverCodecSystem {
    fn build(self, world: &mut World) -> TransceiverCodecSystem {
        let config_path = match application_root_dir() {
            Ok(path) => path.join("config").join("config.ron"),
            Err(err) => panic!(err),
        };

        // TODO: Handle error
        let settings = match TransceiverSettings::load(config_path) {
            Ok(v) => {
                world.insert(v.clone());
                v
            }
            Err(e) => panic!(e),
        };

        let (send, recv): (Sender<Telemetry>, Receiver<Telemetry>) = mpsc::channel();
        let recv = Arc::new(Mutex::new(recv));

        thread::spawn(move || read_serial(settings, send));

        TransceiverCodecSystem {
            trx_recv: Some(recv),
            mag_x_avg: MovingAverage::new(32, None),
            mag_y_avg: MovingAverage::new(32, None),
            gyro_x_avg: MovingAverage::new(16, None),
            gyro_y_avg: MovingAverage::new(16, None),
            gyro_z_avg: MovingAverage::new(16, None),
        }
    }
}

use crate::state::app::CompassUI;
use crate::DroneMarker;
use amethyst::core::timing::Time;

impl<'a> System<'a> for TransceiverCodecSystem {
    // TODO: Create seperate human-readable type for thread
    type SystemData = (
        Read<'a, Option<Arc<Mutex<Receiver<Telemetry>>>>>,
        UiFinder<'a>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Tag<DroneMarker>>,
        Read<'a, Time>,
    );

    fn run(
        &mut self,
        (mut _first, ui_finder, mut ui_text, mut transforms, drones, time): Self::SystemData,
    ) {
        // TODO: Look into .and_then and .map to make this easier to read and more succinct
        let recv = match &self.trx_recv {
            Some(v) => v,
            _ => return,
        };

        let data = match recv.try_lock() {
            Ok(d) => d,
            _ => return,
        };

        let value = match data.try_recv() {
            Ok(v) => v,
            _ => return,
        };

        let mag_x_avg = self.mag_x_avg.add(value.mag_x as f64) as f32;
        let mag_y_avg = self.mag_y_avg.add(value.mag_y as f64) as f32;

        println!("Data: {:?}", value);

        let degrees = crate::compass::coords_to_degrees((mag_x_avg, mag_y_avg));

        let gyro_x =
            value.gyro_x * (std::f32::consts::PI / 180.0) / (1.0 / time.delta_real_seconds());

        let gyro_y =
            value.gyro_z * (std::f32::consts::PI / 180.0) / (1.0 / time.delta_real_seconds());

        let gyro_z =
            value.gyro_y * (std::f32::consts::PI / 180.0) / (1.0 / time.delta_real_seconds());

        for (drone, transform) in (&drones, &mut transforms).join() {
            // println!("Doing a join");
            // transform.set_translation_x(0.5);
            transform.prepend_rotation_x_axis(gyro_x as f32);
            transform.prepend_rotation_y_axis(gyro_y as f32);
            transform.prepend_rotation_z_axis(gyro_z as f32);
            // FIXME: Convert to Vector3 to save two lines
            // transform.append_rotation_x();
            // transform.append_rotation_y();
            // transform.append_rotation_z();
            // transform.
            // transform
        }
        if let Some(heading) = ui_finder
            .find("heading")
            .and_then(|entity| ui_text.get_mut(entity))
        {
            heading.text = format!("{:0padding$.0}", degrees, padding = 3);
        }
    }
}

use crate::cobs_buffer::{Buffer, BufferResult};

fn read_serial(config: TransceiverSettings, send: Sender<Telemetry>) {
    let trx = TransceiverDevice::new((config.vid, config.pid)).unwrap();

    // TODO: Dispatch error if device or multiple are connected
    trx.is_connected().unwrap();

    // TODO: Dispatch error if no serial port is available
    let port_name = trx.port_name().unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.baud_rate = config.baud_rate;

    // TODO: Dispatch error if serial port can not be opened
    let mut port = open_with_settings(&port_name, &settings).unwrap();

    let mut serial_buf: Vec<u8> = vec![0; 256];
    let mut window_buf = Buffer::new();

    loop {
        // TODO: Reduce indentation
        if let Ok(t) = port.read(serial_buf.as_mut_slice()) {
            let mut window = &serial_buf[..t];

            'cobs: while !window.is_empty() {
                use BufferResult::*;

                window = match window_buf.write::<Telemetry>(&window) {
                    Consumed => break 'cobs,
                    Overfull(new_window) => new_window,
                    DeserErr(new_window) => new_window,
                    Success { data, remaining } => {
                        // println!("Data: {:?}", data);

                        send.send(data).unwrap();

                        remaining
                    }
                }
            }
        }
    }
}
