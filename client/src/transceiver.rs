use rusb::Error::{NotFound, NotSupported};
use rusb::{Context, Device, DeviceList, Hotplug, UsbContext};
use serialport::{available_ports, SerialPortType::UsbPort};

struct HotPlugHandler;

impl<T: UsbContext> Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        println!("connected {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("disconnected {:?}", device);
    }
}

pub struct TransceiverDevice {
    context: Context,
    vid: u16,
    pid: u16,
}

impl TransceiverDevice {
    pub fn new((vid, pid): (u16, u16)) -> Result<TransceiverDevice, rusb::Error> {
        let context = Context::new()?;

        Ok(TransceiverDevice { context, vid, pid })
    }

    pub fn port_name(&self) -> Result<String, serialport::ErrorKind> {
        if let Ok(ports) = available_ports() {
            for p in ports {
                match p.port_type {
                    UsbPort(device) => {
                        if (device.vid, device.pid) == (self.vid, self.pid) {
                            return Ok(p.port_name);
                        }
                    }
                    _ => continue,
                }
            }
        }

        Err(serialport::ErrorKind::NoDevice)
    }

    pub fn is_connected(&self) -> rusb::Result<()> {
        for device in DeviceList::new()?.iter() {
            if let Ok(desc) = device.device_descriptor() {
                if desc.vendor_id() == self.vid && desc.product_id() == self.pid {
                    return Ok(());
                }
            }
        }

        Err(NotFound)
    }

    pub fn listen(&self) -> rusb::Result<()> {
        if !rusb::has_hotplug() {
            return Err(NotSupported);
        }

        self.context.register_callback(
            Some(self.vid),
            Some(self.pid),
            None,
            Box::new(HotPlugHandler),
        )?;

        loop {
            self.context.handle_events(None).unwrap()
        }
    }
}
