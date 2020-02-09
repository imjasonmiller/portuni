#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![allow(unused_imports)]
#[allow(unused_extern_crates)]
extern crate embedded_hal;
extern crate panic_itm;
extern crate stm32f30x_hal;

use cortex_m::{iprint, iprintln, peripheral::ITM};
use cortex_m_rt::entry;

use f3::{
    hal::{
        delay::Delay,
        flash::FlashExt,
        gpio::gpioe::{PEx, PE13, PE15},
        gpio::{GpioExt, Output, PushPull},
        i2c::I2c,
        prelude::*,
        rcc::RccExt,
        stm32f30x,
        stm32f30x::i2c1,
        time::U32Ext,
        timer::{Event, Timer},
    },
    l3gd20,
    led::{Direction, Leds},
    lsm303dlhc::{self, MagOdr},
    L3gd20, Lsm303dlhc,
};

use embedded_nrf24l01::{Configuration, CrcMode, DataRate, Error, StandbyMode, NRF24L01};

use postcard::{from_bytes, to_slice_cobs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Telemetry {
    mag_x: i16,
    mag_y: i16,
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
    temp: i8,
}

#[entry]
fn main() -> ! {
    nrf24_tx();
    // nrf24_rx();
}

fn nrf24_tx() -> ! {
    // Cortex and device peripherals
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    // Instrumentation Trace Macrocell for debugging
    // See: https://blog.japaric.io/itm/
    let stim = &mut cp.ITM.stim[0];

    // Split RCC and Flash into different functionalities
    // See: https://blog.japaric.io/brave-new-io/#freezing-the-clock-configuration
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Split GPIO into independent pins and registers
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // LEDs
    let mut led_w: PE15<Output<PushPull>> = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut _led_s: PE13<Output<PushPull>> = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // L3GD20 Gyroscope and temperature sensor
    let l3gd20_nss = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Clocks
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // The `L3gd20` abstraction exposed by the `f3` crate requires a specific pin configuration to
    // be used and won't accept any configuration other than the one used here. Trying to use a
    // different pin configuration will result in a compiler error.
    let l3gd20_sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let l3gd20_miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let l3gd20_mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let l3gd20_spi = f3::hal::spi::Spi::spi1(
        dp.SPI1,
        (l3gd20_sck, l3gd20_miso, l3gd20_mosi),
        l3gd20::MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );
    let mut l3gd20 = L3gd20::new(l3gd20_spi, l3gd20_nss).unwrap();
    l3gd20.set_scale(l3gd20::Scale::Dps500).unwrap();

    // LSM303DLHC Magnetometer and accelerometer
    let lsm303dlhc_scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let lsm303dlhc_sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let lsm303dlhc_i2c = I2c::i2c1(
        dp.I2C1,
        (lsm303dlhc_scl, lsm303dlhc_sda),
        400.khz(),
        clocks,
        &mut rcc.apb1,
    );
    let mut lsm303dlhc = Lsm303dlhc::new(lsm303dlhc_i2c).unwrap();

    // Set the magnetometer output data rate to 30 Hz
    lsm303dlhc.mag_odr(MagOdr::Hz30).unwrap();

    // Configure pins
    let radio_ce = gpiob
        .pb2
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let radio_csn = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let radio_sck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let radio_miso = gpiob.pb14.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let radio_mosi = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

    let radio_spi = f3::hal::spi::Spi::spi2(
        dp.SPI2,
        (radio_sck, radio_miso, radio_mosi),
        embedded_hal::spi::Mode {
            phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
            polarity: embedded_hal::spi::Polarity::IdleLow,
        },
        1.mhz(),
        clocks,
        &mut rcc.apb1,
    );

    let mut radio = NRF24L01::new(radio_ce, radio_csn, radio_spi).unwrap();

    let addr: [u8; 5] = [0x11, 0x11, 0x11, 0x11, 0x11];

    radio.set_frequency(100).unwrap();
    radio.set_tx_addr(&addr).unwrap();
    radio.set_auto_retransmit(0, 0).unwrap();
    radio.set_crc(Some(CrcMode::TwoBytes)).unwrap();
    radio.set_rf(DataRate::R250Kbps, 3).unwrap();
    radio
        .set_auto_ack(&[false, false, false, false, false, false])
        .unwrap();
    radio
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    radio
        .set_pipes_rx_lengths(&[None, Some(1), Some(1), Some(1), Some(1), Some(1)])
        .unwrap();

    radio.flush_tx().unwrap();

    // Transfer into TX
    let mut radio = radio.tx().unwrap();

    // Debug configuration
    iprintln!(stim, "\n");
    iprintln!(stim, "AutoAck: {:?}", radio.get_auto_ack().unwrap());
    iprintln!(stim, "Register: {:?}", radio.get_address_width().unwrap());
    iprintln!(stim, "Frequency: {:?}", radio.get_frequency().unwrap());

    // Delays
    let mut delay = Delay::new(cp.SYST, clocks);

    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    // 32 byte buffer for the NRF24L01+ payload
    let mut buf = [0u8; 32];

    // TX LED Timer
    let mut timer = Timer::tim6(dp.TIM6, 2.hz(), clocks, &mut rcc.apb1);
    let mut is_tx_blinking = false;

    timer.start(2.hz());

    loop {
        // Blink light
        if let Ok(()) = timer.wait() {
            if is_tx_blinking {
                led_w.set_high();
            } else {
                led_w.set_low();
            }

            is_tx_blinking = !is_tx_blinking;
        }

        if let Ok(true) = radio.can_send() {
            // if radio.can_send().unwrap() {
            radio.flush_tx().unwrap();

            // Magnetometer x and y are only needed for heading
            let lsm303dlhc::I16x3 {
                x: mag_x, y: mag_y, ..
            } = lsm303dlhc.mag().unwrap();

            let l3gd20::I16x3 {
                x: gyro_x,
                y: gyro_y,
                z: gyro_z,
            } = l3gd20.gyro().unwrap();

            let gyro_x = l3gd20::Scale::Dps500.degrees(gyro_x);
            let gyro_y = l3gd20::Scale::Dps500.degrees(gyro_y);
            let gyro_z = l3gd20::Scale::Dps500.degrees(gyro_z);

            let temp = l3gd20.temp().unwrap();

            let output = to_slice_cobs(
                &Telemetry {
                    mag_x,
                    mag_y,
                    gyro_x,
                    gyro_y,
                    gyro_z,
                    temp,
                },
                &mut buf,
            )
            .unwrap();

            radio.send(&output).unwrap();
        } else {
            iprintln!(stim, "Cant' send: {}", radio.is_full().unwrap());

            radio.wait_empty().unwrap();
            iprintln!(stim, "Queue is empty");
        }
    }
}

fn nrf24_rx() -> ! {
    // Cortex (cp) and device peripherals (dp)
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    // Instrumentation Trace Macrocell for debugging
    // See: https://blog.japaric.io/itm/
    let stim = &mut cp.ITM.stim[0];

    // Split RCC and Flash into different functionalities
    // See: https://blog.japaric.io/brave-new-io/#freezing-the-clock-configuration
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Split gpiod into independent pins and registers
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // LEDs
    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    // Clocks
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Delays
    let mut delay = Delay::new(cp.SYST, clocks);

    // Configure pins
    let radio_ce = gpiob
        .pb2
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let radio_csn = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let radio_sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let radio_miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let radio_mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let radio_spi = f3::hal::spi::Spi::spi1(
        dp.SPI1,
        (radio_sck, radio_miso, radio_mosi),
        embedded_hal::spi::Mode {
            polarity: embedded_hal::spi::Polarity::IdleLow,
            phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
        },
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut radio = NRF24L01::new(radio_ce, radio_csn, radio_spi).unwrap();

    let addr: [u8; 5] = [0x22, 0x22, 0x22, 0x22, 0x22];

    radio.set_frequency(100).unwrap();
    radio.set_auto_retransmit(0, 0).unwrap();
    radio.set_crc(Some(CrcMode::TwoBytes)).unwrap();
    radio.set_rf(DataRate::R250Kbps, 1).unwrap();
    radio
        .set_auto_ack(&[false, false, false, false, false, false])
        .unwrap();
    radio
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    radio
        .set_pipes_rx_lengths(&[None, Some(1), Some(1), Some(1), Some(1), Some(1)])
        .unwrap();
    radio.set_tx_addr(&addr).unwrap();
    radio.set_rx_addr(0, &addr).unwrap();
    radio.flush_rx().unwrap();
    radio.flush_tx().unwrap();

    delay.delay_ms(10u8);

    // Transfer into RX
    let mut radio = radio.rx().unwrap();

    // Debug configuration
    iprintln!(stim, "\n");
    iprintln!(stim, "AutoAck: {:?}", radio.get_auto_ack().unwrap());
    iprintln!(stim, "Register: {:?}", radio.get_address_width().unwrap());
    iprintln!(stim, "Frequency: {:?}", radio.get_frequency().unwrap());

    delay.delay_us(130u8);

    loop {
        // leds[Direction::South].on();

        // let pipe = radio.can_read().unwrap();

        // if pipe.is_some() {
        // let data = radio.read().unwrap();
        // iprintln!(stim, "Pipe: {:?} Rx: {:?}", pipe, data.as_ref());
        // }

        if let Some(pipe) = radio.can_read().unwrap() {
            iprintln!(stim, "Reading from pipe: {}", pipe);
            let payload = radio.read();
            match payload {
                Ok(p) => {
                    // use core::ops::Deref;
                    // iprintln!(stim, "Payload received: {:?}", p.deref());
                    // let val = u32::from_be_bytes(p.as_ref());

                    // let val = p.as_ref().read_u32::<BigEndian>().unwrap();
                    // iprintln!(stim, "Payload received: {:?}", val);
                    iprintln!(stim, "Payload received: {:?}", p.as_ref());
                    leds[Direction::West].on();
                }
                Err(_) => {
                    iprintln!(stim, "Could not read payload");
                    leds[Direction::North].on();
                }
            }
        }
        leds[Direction::West].off();
        leds[Direction::North].off();
    }
}
