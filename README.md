# ESP32-Garden-Guardian
A Rust-based, bare-metal NixOS project: ESP32 + ULN2003A driving a 5 V relay and pump while reading an Adafruit STEMMA Soil (Seesaw) sensor over I²C.

# 🦀 Project: ESP32 + ULN2003A switches **5 V Relay** & **5 V Pump** + **I²C Soil Moisture Sensor (Adafruit STEMMA Soil / Seesaw)** – Rust + NixOS

## ⚙️ Goal
The **ESP32‑DevKitC** controls
- a **5 V relay module** (low‑active) **and**
- a **5 V pump** (DC motor)  
via a **ULN2003A**, while simultaneously reading a **soil moisture/temperature sensor** (Adafruit **STEMMA Soil**, Seesaw @ **0x36**) over **I²C**.

**Power for the relay/motor side** comes from an **external 5 V power supply**, **not** from ESP32 USB.  
**Sensor power:** **3V3 from the ESP32**, so the Seesaw pull‑ups drive the I²C lines to **3.3 V** (ESP32 GPIOs are **not 5 V‑tolerant**).  
**Software:** Rust with `esp-hal` (new API), flashing via `cargo espflash` on **NixOS**.

---

## 🧱 Hardware Used

| Component | Description | Purpose |
|---|---|---|
| 🧠 ESP32 DevKitC (AZDelivery) | 3.3 V logic, USB‑powered | Main controller |
| ⚙️ ULN2003A (DIP‑16) | 7‑channel Darlington, integrated flyback diodes | GND sink for relay/pump |
| 🔌 5 V relay module (low‑level trigger) | 1‑channel | Switch load |
| 💧 5 V pump (DC) | Motor load | Water delivery |
| 🧪 **Adafruit STEMMA Soil (Seesaw)** | I²C, default address **0x36**, pull‑ups to VIN | Moisture + temperature |
| 🔋 5 V power supply | e.g., ≥2 A | Power for relay + pump |
| 🧰 Breadboard + Dupont + JST‑PH cables | – | Wiring |
| 🔧 Wago clamps, fuse | – | Clean power distribution |

> **Current note:** ULN2003A typically supports **up to 500 mA per channel**. If the pump draws more → use a **relay contact** or **MOSFET**.

---

## 🔌 Pinout & Wiring

### ULN2003A (notch up)
**Left (1..8):** `IN1` … `IN7`, **Pin 8 = GND**  
**Right (9..16):** **Pin 9 = COM**, `OUT7` … **`OUT1 = Pin 16`**

### Control side (ESP32 → ULN)
| ESP32 pin | → | ULN pin | Purpose |
|---|---|---|---|
| **GPIO33** | → | **IN1 (Pin 1)** | Control relay |
| **GPIO32** | → | **IN2 (Pin 2)** | Control pump |
| **GND**    | → | **ULN Pin 8 / breadboard ground** | Common ground |

### Load side (ULN → Relay/Pump)
| ULN pin | → | Target | Purpose |
|---|---|---|---|
| **OUT1 (Pin 16)** | → | **Relay IN** | Switched GND sink for relay |
| **OUT2 (Pin 15)** | → | **Pump negative** | Switched GND sink for pump |
| **COM (Pin 9)** | → | **+5 V (power supply)** | Flyback diodes for motor/relay |

### External 5 V power supply
| Supply | → | Target |
|---|---|---|
| **+5 V** | → | **Relay VCC**, **Pump positive**, optionally breadboard + |
| **GND**  | → | **ULN GND (Pin 8)** / breadboard – |

### Relay (low‑level trigger!)
| Relay pin | → | Target |
|---|---|---|
| **VCC** | → | **+5 V** |
| **GND** | → | **GND** |
| **IN**  | → | **ULN OUT1 (Pin 16)** |

### Pump
| Pump | → | Target |
|---|---|---|
| **Positive** | → | **+5 V** |
| **Negative** | → | **ULN OUT2 (Pin 15)** |

### **I²C Sensor (Adafruit STEMMA Soil / Seesaw)**
> The sensor board has **10 k pull‑ups to VIN** (SDA/SCL). Therefore set **VIN = 3V3** so the I²C lines are at **3.3 V**.

| Sensor pin | → | Target |
|---|---|---|
| **GND** | → | ESP **GND** |
| **VIN** | → | ESP **3V3** |
| **SDA** | → | ESP **GPIO21** |
| **SCL** | → | ESP **GPIO22** |

> **I²C address:** default **0x36**. Via solder jumpers **AD0/AD1** you can add +1/+2 (range **0x36..0x39**).

---

## 🧠 Operating Principle
- **Relay module is low‑active**: IN = GND ⇒ **ON**, IN = High ⇒ **OFF**.  
- **ULN2003A** acts as a **GND switch (open‑collector)**. GPIO = HIGH ⇒ the respective OUT pin sinks to GND.  
- **Flyback diodes:** via **COM → +5 V**, they suppress relay/motor transients. **COM to +5 V is mandatory**.  
- **Seesaw sensor:** I²C commands  
  - **Moisture:** module **0x0F**, function **0x10** ⇒ 16‑bit big‑endian.  
  - **Temperature:** module **0x00**, function **0x04** ⇒ 32‑bit big‑endian, result / **65536**..

# TODO — NixOS prerequisites to flash ESP32/ESP32‑C3 (Rust + espup)

> Scope: **System‑level setup only.** This is **not** about your Rust project code.  
> Goal: You can build/flash from any ESP Rust repo without manual, ad‑hoc steps.

---

## 0) Assumptions
- User account: **USER** (adjust if different).
- You use **rustup** + **espup** (custom toolchain `esp`) and `cargo-espflash`.
- Serial adapter appears as `/dev/ttyUSB0` (CH340/CP210x/etc.).

---

## 1) NixOS module (one‑time)

Add `/etc/nixos/modules/rust.nix` and import it from `configuration.nix`:

```nix
{ config, lib, pkgs, ... }:
let
  user = "USER";

  # One‑shot script to *create/refresh* the ESP toolchain and ~/export-esp.sh
  espBootstrap = pkgs.writeShellScript "esp-bootstrap" ''
    set -euo pipefail
    export HOME="/home/${user}"
    export PATH="$HOME/.cargo/bin:${pkgs.rustup}/bin:${pkgs.coreutils}/bin:${pkgs.bash}/bin:${pkgs.gnugrep}/bin"

    # 1) Ensure custom toolchain 'esp' exists (downloads via espup)
    if ! ${pkgs.rustup}/bin/rustup toolchain list | ${pkgs.gnugrep}/bin/grep -q '^esp'; then
      echo "[esp-bootstrap] Installing ESP toolchain via espup …"
      ${pkgs.espup}/bin/espup install --export-file "$HOME/export-esp.sh"
    fi

    # 2) Ensure export file exists (this is what your shell sources)
    if [ ! -s "$HOME/export-esp.sh" ]; then
      echo "[esp-bootstrap] Recreating export-esp.sh via espup …"
      ${pkgs.espup}/bin/espup install --export-file "$HOME/export-esp.sh"
    fi
  '';
in {
  # Serial access
  users.users.${user}.extraGroups = [ "dialout" ];

  # Tools you need at the system level (no pkgs.cargo here — cargo comes from rustup)
  environment.systemPackages = with pkgs; [
    rustup espup cargo-espflash esptool
    cmake ninja pkg-config gnumake gcc
    llvmPackages_16.libclang
    python3 libusb1
  ];

  # Runtime libs for foreign binaries (bindgen/libclang, esptool, etc.)
  programs.nix-ld.enable = true;
  programs.nix-ld.libraries = with pkgs; [
    stdenv.cc.cc.lib
    zlib libffi libxml2
    libusb1
  ];

  # Shell env for USER — sources the esp exports *if present*
  environment.interactiveShellInit = ''
    export PATH="$HOME/.cargo/bin:$PATH"
    [ -f "$HOME/export-esp.sh" ] && . "$HOME/export-esp.sh"

    # Make bindgen find system libclang on NixOS
    export LIBCLANG_PATH="${pkgs.llvmPackages_16.libclang.lib}/lib"

    # Conservative runtime set for bindgen/libclang
    export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libxml2}/lib:${pkgs.zlib}/lib:${pkgs.libffi}/lib"

    # Avoid char‑signedness headaches in some ESP bindings
    export BINDGEN_EXTRA_CLANG_ARGS="-fsigned-char"
    export BINDGEN_EXTRA_CLANG_ARGS_xtensa_esp32_espidf="-fsigned-char"
  '';

  # System‑level one‑shot that runs as 'USER' and *creates* ~/export-esp.sh
  systemd.services.espup-bootstrap = {
    description = "Ensure ESP Rust toolchain and ~/export-esp.sh for ${user}";
    wantedBy    = [ "multi-user.target" ];
    after       = [ "network-online.target" ];
    wants       = [ "network-online.target" ];  # actually require network for downloads
    serviceConfig = {
      Type = "oneshot";
      User = user;
      Environment = [ "HOME=/home/${user}" ];
      ExecStart = "${espBootstrap}";
    };
  };
}
```

Then apply:
```bash
sudo nixos-rebuild switch
```

(Optional) Kick the bootstrap right away:
```bash
sudo systemctl start espup-bootstrap.service
sudo systemctl status espup-bootstrap.service
```

---

## 1a) The `~/export-esp.sh` file (explicit)

- **What it is:** A small shell script that sets PATH/LIBCLANG_PATH for the ESP toolchain.  
- **Who creates it:** **`espup install --export-file "$HOME/export-esp.sh"`** (our systemd oneshot does that).  
- **Do not commit it:** Paths include **machine‑local** toolchain versions; they change over time.

**Example layout (your actual versions will differ):**
```bash
# ~/.cargo/bin & toolchain bins must come first
export PATH="/home/USER/.rustup/toolchains/esp/xtensa-esp-elf/esp-15.2.0_YYYYMMDD/xtensa-esp-elf/bin:$PATH"
# libclang for ESP (clang‑based IDF builds, bindgen, etc.)
export LIBCLANG_PATH="/home/USER/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-20.1.1_YYYYMMDD/esp-clang/lib"
```

**Concrete example from a working machine:**
```bash
export PATH="/home/marvin/.rustup/toolchains/esp/xtensa-esp-elf/esp-15.2.0_20250920/xtensa-esp-elf/bin:$PATH"
export LIBCLANG_PATH="/home/marvin/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-20.1.1_20250829/esp-clang/lib"
```

If versions bump later, simply re‑run:
```bash
espup install --export-file "$HOME/export-esp.sh"
```

---

## 2) Verify the *dependencies* (not your project)

- Toolchain and export file:
  ```bash
  rustup toolchain list | grep '^esp'     # 'esp' must be listed
  test -s "$HOME/export-esp.sh" && echo "export-esp.sh: OK"
  ```

- PATH sanity (cargo must come from rustup):
  ```bash
  which cargo espup espflash
  # expect: cargo -> ~/.cargo/bin/cargo
  ```

- Serial permissions (after adding you to 'dialout' either re-login or run `newgrp dialout`):
  ```bash
  ls -l /dev/ttyUSB0 | awk '{print $1,$3,$4,$9}'
  # expect group 'dialout'
  ```

- Targets installed for the custom toolchain:
  ```bash
  rustup +esp target list --installed
  ```

---

## 3) Build/flash (only to prove the deps work)

> Minimal smoke test to prove the environment is correct.

```bash
# In your project folder:
. "$HOME/export-esp.sh"              # ensures the esp toolchain env is loaded
cargo +esp build --release

# Flash (auto-detect is usually fine):
cargo espflash flash --release --monitor --port /dev/ttyUSB0

# Force chip if needed:
# cargo espflash flash --chip esp32   --release --monitor --port /dev/ttyUSB0
# cargo espflash flash --chip esp32c3 --release --monitor --port /dev/ttyUSB0
```

---

## 4) Troubleshooting (dependency‑level)

- **`esp` missing** → Run the bootstrap:  
  ```bash
  sudo systemctl start espup-bootstrap
  espup install --export-file "$HOME/export-esp.sh"   # manual fallback
  ```

- **Wrong cargo** (`/run/current-system/sw/bin/cargo`) → Remove `pkgs.cargo` from your NixOS config; rely on rustup.

- **`/dev/ttyUSB0` permission denied** → Re‑login or `newgrp dialout`.

- **“ordered after network-online… but doesn’t depend”** → We set `wants = [ "network-online.target" ]` to both silence the warning and actually wait for network.

- **Serial port busy** → Close VSCode serial, `screen`, `miniterm`, etc.

---

## 5) Out of scope (by design)
- Your repo’s `rust-toolchain.toml` (use `channel = "esp"` if you rely on espup’s toolchain).
- Project code, Cargo features, board‑specific GPIO pinouts, etc.

**Bottom line:** This module **creates** `~/export-esp.sh` via `espup`, ensures the `esp` toolchain exists, sets sane env/paths, and gives you the rights to talk to the USB serial device. After that, *any* ESP Rust project should build/flash without ad‑hoc fixes.
