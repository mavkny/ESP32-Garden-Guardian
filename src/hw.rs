use anyhow::Result;
use esp_idf_hal::gpio::{Gpio21, Gpio22, Gpio33, Output, PinDriver, Pins};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::Hertz;
use crate::config::RELAY_ACTIVE_LOW;

///  - I2C: SDA=GPIO21, SCL=GPIO22
///  - Relais: GPIO33 (Pumpe läuft über Relais-Kontakt NO/COM)

pub fn init_hw(
    pins: Pins,
    i2c0: I2C0,
    i2c_hz: u32,
) -> Result<(I2cDriver<'static>, PinDriver<'static, Gpio33, Output>)> {
    // I2C
    let sda: Gpio21 = pins.gpio21;
    let scl: Gpio22 = pins.gpio22;
    let i2c = I2cDriver::new(
        i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(Hertz(i2c_hz.max(1))),
    )?;

    // Relais GPIO (Pumpe läuft über Relais-Kontakt)
    let mut relay = PinDriver::output(pins.gpio33)?;

    // Relais initial AUS
    if RELAY_ACTIVE_LOW { relay.set_high()?; } else { relay.set_low()?; }

    Ok((i2c, relay))
}

