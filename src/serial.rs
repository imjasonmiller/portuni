extern crate serialport;

use serialport::prelude::*;
use serialport::SerialPortType;
use std::io;
use std::io::ErrorKind;

use std::prelude::*;

fn find_port(vid: u16, pid: u16) -> Result<String, ErrorKind> {
    if let Ok(ports) = serialport::available_ports() {
        let mut port_name: String;

        for p in ports {
            match p.port_type {
                SerialPortType::UsbPort(info) => {
                    let pair = (info.vid, info.pid);

                    match pair {
                        (2341, 0043) => {
                            return Ok(p.port_name);
                        }
                        _ => return Err(ErrorKind::NotFound),
                    }
                }
                _ => return Err(ErrorKind::NotFound),
            }
        }

        Ok("test".to_string())
    } else {
        return Err(ErrorKind::NotFound);
    }
}

fn main() {
    let port = find_port(2341, 0043).unwrap();

    println!("port: {}", port);
}
