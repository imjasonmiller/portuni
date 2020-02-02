use rusb::Error::{NotFound, NotSupported};
use rusb::{Context, Device, DeviceList, UsbContext};

struct HotPlugHandler;

impl<T: UsbContext> rusb::Hotplug<T> for HotPlugHandler {
    fn device_arrived(&mut self, device: Device<T>) {
        println!("connected {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("disconnected {:?}", device);
    }
}

pub struct Receiver {
    context: Context,
    vid: u16,
    pid: u16,
}

impl Receiver {
    pub fn new((vid, pid): (u16, u16)) -> Result<Receiver, rusb::Error> {
        let context = Context::new()?;

        Ok(Receiver { context, vid, pid })
    }

    pub fn is_connected(&self) -> rusb::Result<()> {
        for device in DeviceList::new()?.iter() {
            let device_desc = match device.device_descriptor() {
                Ok(d) => d,
                Err(_) => continue,
            };

            if device_desc.vendor_id() == self.vid && device_desc.product_id() == self.pid {
                // Listen
                // println!("{:?}", device.)
                return Ok(());
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
