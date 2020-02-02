extern crate serialport;

use serialport::prelude::*;
use serialport::SerialPortType;
use std::io;
use std::io::ErrorKind;

use std::prelude::*;

pub fn serial_port((vid, pid): (u16, u16)) -> Result<String, serialport::ErrorKind> {
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
