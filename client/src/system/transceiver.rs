use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

use amethyst::{
    config::Config,
    core::SystemDesc,
    ecs::prelude::{Read, ReadExpect, System, SystemData, Write, WriteStorage},
    prelude::*,
    ui::{UiFinder, UiText},
    utils::application_root_dir,
};

use serialport::{open_with_settings, SerialPortSettings};

use crate::config::TransceiverSettings;
use crate::transceiver::TransceiverDevice;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Magnetometer {
    x: i16,
    y: i16,
    command: String,
}

pub struct TransceiverCodecSystem {
    trx_recv: Option<Arc<Mutex<Receiver<Magnetometer>>>>,
}

impl TransceiverCodecSystem {
    pub fn new() -> TransceiverCodecSystem {
        TransceiverCodecSystem { trx_recv: None }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, TransceiverCodecSystem> for TransceiverCodecSystem {
    fn build(self, world: &mut World) -> TransceiverCodecSystem {
        let config_path = match application_root_dir() {
            Ok(path) => path.join("config").join("config.ron"),
            Err(err) => panic!(err),
        };

        let config = TransceiverSettings::load(config_path).unwrap();
        let config2 = config.clone();
        world.insert(config);

        let (send, recv): (Sender<Magnetometer>, Receiver<Magnetometer>) = mpsc::channel();

        let recv = Arc::new(Mutex::new(recv));

        thread::spawn(move || read_serial(send, config2));

        TransceiverCodecSystem {
            trx_recv: Some(recv),
        }
    }
}

use crate::state::app::CompassUI;

impl<'a> System<'a> for TransceiverCodecSystem {
    // TODO: Create seperate human-readable type for thread
    type SystemData = (
        Read<'a, Option<Arc<Mutex<Receiver<Magnetometer>>>>>,
        UiFinder<'a>,
        WriteStorage<'a, UiText>,
        Read<'a, CompassUI>,
    );

    // fn run(&mut self, _data: Self::SystemData) {
    fn run(&mut self, (mut _first, ui_finder, mut ui_text, _compass_ui): Self::SystemData) {
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

        let degrees = crate::compass::coords_to_degrees((value.x as f32, value.y as f32));

        if let Some(heading) = ui_finder
            .find("heading")
            .and_then(|entity| ui_text.get_mut(entity))
        {
            heading.text = format!("{:0padding$.0}", degrees, padding = 3);
        }
    }
}

use crate::cobs_buffer::Buffer;
use crate::cobs_buffer::BufferResult;

fn read_serial(send: Sender<Magnetometer>, config: TransceiverSettings) {
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

                window = match window_buf.write::<Magnetometer>(&window) {
                    Consumed => break 'cobs,
                    Overfull(new_window) => new_window,
                    DeserErr(new_window) => new_window,
                    Success { data, remaining } => {
                        send.send(data).unwrap();

                        remaining
                    }
                }
            }
        }
    }
}
