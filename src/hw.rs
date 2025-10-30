use anyhow::Result;
use esp_idf_hal::gpio::{Gpio21, Gpio22, Gpio32, Gpio33, Output, PinDriver, Pins};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver, I2C0};
use esp_idf_hal::units::Hertz;
use crate::config::{I2C_HZ, RELAY_ACTIVE_LOW, PUMP_ACTIVE_LOW};

///  - I2C: SDA=GPIO21, SCL=GPIO22
///  - Relais: GPIO33
///  - Pump:  GPIO32

pub fn init_hw(
    pins: Pins,
    i2c0: I2C0,
    i2c_hz: u32,
) -> Result<(I2cDriver<'static>, PinDriver<'static, Gpio33, Output>, PinDriver<'static, Gpio32, Output>)> {
    // I2C
    let sda: Gpio21 = pins.gpio21;
    let scl: Gpio22 = pins.gpio22;
    let i2c = I2cDriver::new(
        i2c0,
        sda,
        scl,
        &I2cConfig::new().baudrate(Hertz(i2c_hz.max(1))),
    )?;

    // GPIOs
    let mut relay = PinDriver::output(pins.gpio33)?;
    let mut pump  = PinDriver::output(pins.gpio32)?;

    if RELAY_ACTIVE_LOW { relay.set_high()?; } else { relay.set_low()?; }
    if PUMP_ACTIVE_LOW  { pump.set_high()?;  } else { pump.set_low()?;  }

    Ok((i2c, relay, pump))
}

