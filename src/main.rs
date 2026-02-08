mod credentials;  // Auto-generiert aus credentials.toml
mod config;
mod sensor;
mod control;
mod hw;
mod web;

use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use config::Reading;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration as WifiCfg, EspWifi};
// mDNS temporär deaktiviert (braucht ESP-IDF Komponente)
// use esp_idf_svc::mdns::EspMdns;
use esp_idf_sys as _;

// const MDNS_HOSTNAME: &str = "kneplant";

fn main() -> Result<()> {
    EspLogger::initialize_default();

    let _handle = thread::Builder::new()
        .name("app".into())
        .stack_size(28 * 1024)
        .spawn(|| {
            if let Err(e) = app() {
                log::error!("app() failed: {e:?}");
            }
        })?;

    loop { thread::sleep(Duration::from_secs(60)); }
}

fn app() -> Result<()> {
    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let modem = peripherals.modem;
    let i2c0 = peripherals.i2c0;

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop.clone())?;

    wifi.set_configuration(&WifiCfg::Client(ClientConfiguration {
        ssid: config::WIFI_SSID.try_into().unwrap(),
        password: config::WIFI_PASS.try_into().unwrap(),
        ..Default::default()
    }))?;
    wifi.start()?;
    unsafe { esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE); }
    wifi.connect()?;
    wifi.wait_netif_up()?;

    // IP über das Wrapper-Objekt (kein Parallel-Borrow von esp_wifi)
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("WiFi connected. IP: {}", ip_info.ip);

    // mDNS temporär deaktiviert (braucht ESP-IDF mdns Komponente)
    // TODO: ESP_IDF_COMPONENTS="mdns" aktivieren für kneplant.local
    // let mut mdns = EspMdns::take()?;
    // mdns.set_hostname(MDNS_HOSTNAME)?;
    // mdns.set_instance_name("KnePlant Garden Guardian")?;
    // mdns.add_service(None, "_http", "_tcp", 80, &[])?;
    // log::info!("mDNS: http://{}.local", MDNS_HOSTNAME);

    // HW
    let (i2c, relay) = hw::init_hw(pins, i2c0, config::I2C_HZ)?;

    // Shared State
    let state = Arc::new(Mutex::new(Reading {
        moisture: 0,
        temp_c: f32::NAN,
        pump_on: false,
        last_pump_us: None,
    }));

    let ctrl_tx = control::spawn_control(i2c, relay, state.clone())?;
    let _server = web::start_web(state, ctrl_tx)?;

    // Reconnect-„Watchdog“ im selben Thread (keine 'static-Lifetime nötig)
    loop {
        if !wifi.is_connected().unwrap_or(false) {
            let _ = wifi.connect();
            let _ = wifi.wait_netif_up();
            if let Ok(info) = wifi.wifi().sta_netif().get_ip_info() {
                log::warn!("WiFi reconnected. IP: {}", info.ip);
            } else {
                log::warn!("WiFi reconnected.");
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

