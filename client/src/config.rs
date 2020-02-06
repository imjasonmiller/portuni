use serde::{Deserialize, Serialize};
use std::time::Duration;

mod shim_baud_rate {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &u32, s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
        match u32::deserialize(d)? {
            9_600 => Ok(9_600),
            14_400 => Ok(14_400),
            19_200 => Ok(19_200),
            38_400 => Ok(38_400),
            57_600 => Ok(57_600),
            115_200 => Ok(115_200),
            n => Err(D::Error::custom(format_args!(
                "Invalid baud_rate value {}",
                n
            ))),
        }
    }
}

fn default_baud_rate() -> u32 {
    9600
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", remote = "serialport::FlowControl")]
enum FlowControlDef {
    None,
    Software,
    Hardware,
}

fn default_flow_control() -> serialport::FlowControl {
    serialport::FlowControl::None
}

mod shim_data_bits {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use serialport::DataBits;
    use serialport::DataBits::*;

    pub fn serialize<S: Serializer>(v: &DataBits, s: S) -> Result<S::Ok, S::Error> {
        let v: u8 = match v {
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
        };

        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DataBits, D::Error> {
        match u8::deserialize(d)? {
            5 => Ok(Five),
            6 => Ok(Six),
            7 => Ok(Seven),
            8 => Ok(Eight),
            n => Err(D::Error::custom(format_args!(
                "Invalid data_bits value {}",
                n
            ))),
        }
    }
}

fn default_data_bits() -> serialport::DataBits {
    serialport::DataBits::Eight
}

mod shim_stop_bits {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use serialport::StopBits;
    use serialport::StopBits::*;

    pub fn serialize<S: Serializer>(v: &StopBits, s: S) -> Result<S::Ok, S::Error> {
        let v: u8 = match v {
            One => 1,
            Two => 6,
        };

        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<StopBits, D::Error> {
        match u8::deserialize(d)? {
            1 => Ok(One),
            2 => Ok(Two),
            n => Err(D::Error::custom(format_args!(
                "Invalid stop_bits value {}",
                n
            ))),
        }
    }
}

fn default_stop_bits() -> serialport::StopBits {
    serialport::StopBits::One
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", remote = "serialport::Parity")]
enum ParityDef {
    None,
    Odd,
    Even,
}

fn default_parity() -> serialport::Parity {
    serialport::Parity::None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TransceiverSettings {
    pub vid: u16,
    pub pid: u16,

    #[serde(default = "default_baud_rate", with = "shim_baud_rate")]
    pub baud_rate: u32,

    #[serde(default = "default_flow_control", with = "FlowControlDef")]
    pub flow_control: serialport::FlowControl,

    #[serde(default = "default_data_bits", with = "shim_data_bits")]
    pub data_bits: serialport::DataBits,

    #[serde(default = "default_stop_bits", with = "shim_stop_bits")]
    pub stop_bits: serialport::StopBits,

    #[serde(default = "default_parity", with = "ParityDef")]
    pub parity: serialport::Parity,

    pub timeout: Duration,
}

impl Default for TransceiverSettings {
    fn default() -> Self {
        Self {
            vid: 0x2341,
            pid: 0x0043,
            baud_rate: 115_200,
            flow_control: serialport::FlowControl::None,
            data_bits: serialport::DataBits::Eight,
            stop_bits: serialport::StopBits::One,
            parity: serialport::Parity::None,
            timeout: Duration::from_millis(10),
        }
    }
}
