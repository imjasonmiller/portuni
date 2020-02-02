mod cobs_buffer;
mod compass;
mod config;
mod usbreader;

use amethyst::config::Config;
use amethyst::utils::application_root_dir;

use crate::cobs_buffer::*;
use crate::config::ReceiverSettings;
use usbreader::Receiver;

use serde::{Deserialize, Serialize};
use serialport::{SerialPortSettings, SerialPortType};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Magnetometer {
    x: i16,
    y: i16,
    command: String,
}

fn serial_port((vid, pid): (u16, u16)) -> Result<String, serialport::ErrorKind> {
    if let Ok(ports) = serialport::available_ports() {
        for p in ports {
            match p.port_type {
                SerialPortType::UsbPort(device) => {
                    if (device.vid, device.pid) == (vid, pid) {
                        return Ok(p.port_name);
                    }
                }
                _ => continue,
            }
        }
    }

    Err(serialport::ErrorKind::NoDevice)
}

fn main() -> amethyst::Result<()> {
    let app_root = application_root_dir()?;
    let config_dir = app_root.join("config");
    let config_path = config_dir.join("config.ron");
    let rx_settings = ReceiverSettings::load(&config_path)?;

    let rx = Receiver::new((rx_settings.vid, rx_settings.pid))?;

    match rx.is_connected() {
        Ok(_r) => {
            let port_name = serial_port((rx_settings.vid, rx_settings.pid)).unwrap();
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
