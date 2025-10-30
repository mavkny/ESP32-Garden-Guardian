use anyhow::Result;
use crate::config::{self, Reading, RELAY_ACTIVE_LOW, PUMP_ACTIVE_LOW};
use crate::sensor;
use esp_idf_hal::gpio::{Gpio32, Gpio33, Output, PinDriver};
use esp_idf_hal::i2c::I2cDriver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub enum ControlCmd {
    /// Pumpe für N Sekunden aktivieren
    ManualPump(u64),
}

fn now_us() -> u64 {
    unsafe { esp_idf_sys::esp_timer_get_time() as u64 }
}

#[inline]
fn set_with_polarity(pin: &mut PinDriver<Gpio32, Output>, on: bool, active_low: bool) -> anyhow::Result<()> {
    if active_low {
        if on { pin.set_low()?; } else { pin.set_high()?; }
    } else {
        if on { pin.set_high()?; } else { pin.set_low()?; }
    }
    Ok(())
}

#[inline]
fn set_relay(relay: &mut PinDriver<Gpio33, Output>, on: bool) -> anyhow::Result<()> {
    if RELAY_ACTIVE_LOW {
        if on { relay.set_low()?; } else { relay.set_high()?; }
    } else {
        if on { relay.set_high()?; } else { relay.set_low()?; }
    }
    Ok(())
}

fn run_pump(
    relay: &mut PinDriver<Gpio33, Output>,
    pump: &mut PinDriver<Gpio32, Output>,
    secs: u64,
    state: &Arc<Mutex<Reading>>,
) {
    // Einschalten
    let _ = set_relay(relay, true);
    let _ = set_with_polarity(pump, true, PUMP_ACTIVE_LOW);
    {
        let mut s = state.lock().unwrap();
        s.pump_on = true;
        s.last_pump_us = Some(now_us());
    }

    // feste Laufzeit
    thread::sleep(Duration::from_secs(secs));

    // Sicher ausschalten
    let _ = set_with_polarity(pump, false, PUMP_ACTIVE_LOW);
    let _ = set_relay(relay, false);
    {
        let mut s = state.lock().unwrap();
        s.pump_on = false;
    }
}

/// Startet den Steuer-Thread. Liefert einen Sender für manuelle Pump-Befehle.
pub fn spawn_control(
    mut i2c: I2cDriver<'static>,
    mut relay: PinDriver<'static, Gpio33, Output>,
    mut pump:  PinDriver<'static, Gpio32, Output>,
    state: Arc<Mutex<Reading>>,
) -> Result<mpsc::Sender<ControlCmd>> {

    // Initial AUS anhand der Polarität
    let _ = set_relay(&mut relay, false);
    let _ = set_with_polarity(&mut pump, false, PUMP_ACTIVE_LOW);

    let (tx, rx) = mpsc::channel::<ControlCmd>();

    thread::Builder::new()
    .name("control".into())
    .stack_size(8 * 1024)
    .spawn(move || {
        let mut last_sensor = Instant::now() - Duration::from_secs(10);

        loop {
            // 1) Kommandos mit Timeout behandeln
            if let Ok(cmd) = rx.recv_timeout(Duration::from_millis(50)) {
                match cmd {
                    ControlCmd::ManualPump(secs) => {
                        run_pump(&mut relay, &mut pump, secs, &state);
                    }
                }
            }

            // 2) Sensoren alle ~1s lesen
            if last_sensor.elapsed() >= Duration::from_secs(1) {
                last_sensor = Instant::now();
                let moist_opt = sensor::avg_moisture(&mut i2c, 3).ok();
                let temp_opt  = sensor::read_temp_c(&mut i2c).ok();

                // State aktualisieren
                let (mut moist_val, mut pump_on_now) = (None, false);
                {
                    let mut s = state.lock().unwrap();
                    if let Some(m) = moist_opt { s.moisture = m; moist_val = Some(m); }
                    if let Some(t) = temp_opt  { s.temp_c = t; }
                    pump_on_now = s.pump_on;
                }

                // 3) Automatik: Feuchte <= Threshold -> Pumpe für AUTO_PUMP_SECS
                if config::AUTO_PUMP_ENABLE {
                    if let Some(m) = moist_val {
                        if !pump_on_now && m <= config::AUTO_MOISTURE_THRESHOLD {
                            run_pump(&mut relay, &mut pump, config::AUTO_PUMP_SECS, &state);
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    })?;

    Ok(tx)
}
