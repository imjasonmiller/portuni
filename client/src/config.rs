use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[serde(remote = "serialport::DataBits")]
enum DataBitsDef {
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
}

fn default_data_bits() -> serialport::DataBits {
    serialport::DataBits::Eight
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

#[derive(Serialize, Deserialize)]
#[serde(remote = "serialport::StopBits")]
enum StopBitsDef {
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
}

fn default_stop_bits() -> serialport::StopBits {
    serialport::StopBits::One
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ReceiverSettings {
    pub vid: u16,
    pub pid: u16,

    #[serde(default)]
    pub baud_rate: u32,

    #[serde(default = "default_data_bits", with = "DataBitsDef")]
    pub data_bits: serialport::DataBits,

    #[serde(default = "default_flow_control", with = "FlowControlDef")]
    pub flow_control: serialport::FlowControl,

    #[serde(default = "default_parity", with = "ParityDef")]
    pub parity: serialport::Parity,

    #[serde(default = "default_stop_bits", with = "StopBitsDef")]
    pub stop_bits: serialport::StopBits,

    pub timeout: Duration,
}

impl Default for ReceiverSettings {
    fn default() -> ReceiverSettings {
        ReceiverSettings {
            vid: 0x2341,
            pid: 0x0043,
            baud_rate: 115_200,
            flow_control: serialport::FlowControl::None,
            data_bits: serialport::DataBits::Eight,
            stop_bits: serialport::StopBits::One,
            parity: serialport::Parity::None,
            timeout: Duration::from_secs(1),
        }
    }
}
