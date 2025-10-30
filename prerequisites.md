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
