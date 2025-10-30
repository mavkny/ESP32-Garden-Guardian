use crate::config::ADDR;
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::i2c::I2cDriver;
use std::thread;
use std::time::Duration;

pub fn avg_moisture(i2c: &mut I2cDriver<'static>, n: usize) -> Result<u16, ()> {
    let mut acc: u32 = 0;
    let mut cnt: u32 = 0;
    for _ in 0..n {
        match read_moisture(i2c) {
            Ok(m) => { acc += m as u32; cnt += 1; }
            Err(_) => { /* Sample verwerfen */ }
        }
        thread::sleep(Duration::from_millis(10));
    }
    if cnt == 0 { return Err(()); }
    Ok((acc / cnt) as u16)
}

/// Seesaw: CAP 0x0F/0x10 -> 16-bit BE
pub fn read_moisture(i2c: &mut I2cDriver<'static>) -> Result<u16, ()> {
    i2c.write(ADDR, &[0x0F, 0x10], BLOCK).map_err(|_| ())?;
    thread::sleep(Duration::from_millis(10));
    let mut buf = [0u8; 2];
    i2c.read(ADDR, &mut buf, BLOCK).map_err(|_| ())?;
    Ok(u16::from_be_bytes(buf))
}

/// Seesaw: STATUS 0x00/TEMP 0x04 -> 32-bit BE, /65536
pub fn read_temp_c(i2c: &mut I2cDriver<'static>) -> Result<f32, ()> {
    i2c.write(ADDR, &[0x00, 0x04], BLOCK).map_err(|_| ())?;
    thread::sleep(Duration::from_millis(10));
    let mut buf = [0u8; 4];
    i2c.read(ADDR, &mut buf, BLOCK).map_err(|_| ())?;
    Ok(u32::from_be_bytes(buf) as f32 / 65_536.0)
}

