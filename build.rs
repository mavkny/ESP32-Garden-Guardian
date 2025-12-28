use embuild::{build::CfgArgs, espidf};
use std::fs;
use std::path::Path;

fn main() {
    CfgArgs::output_propagated("ESP_IDF").unwrap();
    espidf::sysenv::output();

    println!("cargo:rerun-if-changed=sdkconfig.defaults");
    println!("cargo:rerun-if-changed=partitions_singleapp.csv");
    println!("cargo:rerun-if-changed=credentials.toml");

    // Lade WiFi-Credentials aus credentials.toml
    let creds_path = Path::new("credentials.toml");
    if !creds_path.exists() {
        panic!(
            "\n\n\
            ========================================\n\
            ERROR: credentials.toml nicht gefunden!\n\
            \n\
            Kopiere credentials.toml.example zu credentials.toml\n\
            und trage deine WiFi-Daten ein.\n\
            ========================================\n\n"
        );
    }

    let content = fs::read_to_string(creds_path).expect("Kann credentials.toml nicht lesen");
    let config: toml::Value = content.parse().expect("Ungültiges TOML in credentials.toml");

    let wifi = config.get("wifi").expect("Kein [wifi] Abschnitt in credentials.toml");
    let ssid = wifi
        .get("ssid")
        .and_then(|v| v.as_str())
        .expect("wifi.ssid fehlt");
    let password = wifi
        .get("password")
        .and_then(|v| v.as_str())
        .expect("wifi.password fehlt");

    // Generiere src/credentials.rs
    let out_path = Path::new("src/credentials.rs");
    let generated = format!(
        r#"// AUTO-GENERATED - NICHT EDITIEREN!
// Generiert aus credentials.toml durch build.rs

pub const WIFI_SSID: &str = "{}";
pub const WIFI_PASS: &str = "{}";
"#,
        ssid.escape_default(),
        password.escape_default()
    );

    fs::write(out_path, generated).expect("Kann src/credentials.rs nicht schreiben");
}
