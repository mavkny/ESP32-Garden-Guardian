use core::f32;

// WiFi-Credentials werden aus credentials.toml geladen (build.rs → src/credentials.rs)
pub use crate::credentials::{WIFI_SSID, WIFI_PASS};

/*** Hardware & Sensor ***/
pub const I2C_HZ: u32 = 400_000;
pub const ADDR: u8 = 0x36; // STEMMA Soil / Seesaw default

// Relais-Polaritäten getrennt konfigurierbar:
// Mit ULN2003A dazwischen: GPIO HIGH → ULN sinkt → Relais-IN = GND → Relais AN
// Also: RELAY_ACTIVE_LOW = false (GPIO HIGH = Relais AN)
pub const RELAY_ACTIVE_LOW: bool = false;  // Relais über ULN (invertiert!)
// PUMP_ACTIVE_LOW entfernt - Pumpe läuft jetzt über Relais-Kontakt

// Dauer für den manuellen Pumpstoß (Sekunden)
pub const MANUAL_PUMP_SECS: u64 = 5;

/*** Automatik ***/
pub const AUTO_PUMP_ENABLE: bool = true;

// Hysterese: Pumpe AN wenn <= LOW, AUS-Bedingung wenn >= HIGH
pub const AUTO_MOISTURE_LOW: u16 = 750;   // Pumpe an wenn <= 750 (trocken)
pub const AUTO_MOISTURE_HIGH: u16 = 850;  // Erst wieder prüfen wenn >= 850 (feucht genug)

pub const AUTO_PUMP_SECS: u64 = 7;        // Laufzeit in Sekunden
pub const AUTO_COOLDOWN_SECS: u64 = 300;  // 5 Minuten Pause nach Pumpe

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
