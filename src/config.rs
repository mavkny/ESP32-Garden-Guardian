use core::f32;

pub const WIFI_SSID: &str = "SSID";
pub const WIFI_PASS: &str = "PASSWORD!";

/*** Hardware & Sensor ***/
pub const I2C_HZ: u32 = 400_000;
pub const ADDR: u8 = 0x36; // STEMMA Soil / Seesaw default

// Relais-Polaritäten getrennt konfigurierbar:
// true  => aktiv bei LOW (low-trigger, OFF = HIGH)
// false => aktiv bei HIGH (aktiv-high,   OFF = LOW)
pub const RELAY_ACTIVE_LOW: bool = true;   // Relais
pub const PUMP_ACTIVE_LOW:  bool = false;  // Pumpe (bei LOW offline => aktiv-high)

// Dauer für den manuellen Pumpstoß (Sekunden)
pub const MANUAL_PUMP_SECS: u64 = 5;

/*** Automatik ***/
pub const AUTO_PUMP_ENABLE: bool = true;
pub const AUTO_MOISTURE_THRESHOLD: u16 = 670; // <= 670 -> Pumpe an
pub const AUTO_PUMP_SECS: u64 = 7;            // Laufzeit in Sekunden

/*** App-Model ***/
#[derive(Clone, Debug)]
pub struct Reading {
    pub moisture: u16,
    pub temp_c: f32,
    /// Nur Info, ob der Pump-Pin gerade aktiv ist
    pub pump_on: bool,
    /// µs seit Boot (esp_timer_get_time) der letzten Aktivierung (manuell/auto)
    pub last_pump_us: Option<u64>,
}
