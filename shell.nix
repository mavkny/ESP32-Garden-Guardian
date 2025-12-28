{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "esp32-dev";

  buildInputs = with pkgs; [
    # Rust via rustup (nicht system cargo!)
    rustup

    # ESP-IDF Tools
    espup
    espflash
    esptool

    # Build Dependencies
    cmake
    ninja
    pkg-config
    gnumake
    gcc

    # LLVM/Clang für bindgen
    llvmPackages_18.libclang

    # Python für esptool
    python3

    # USB
    libusb1
  ];

  # Für nix-ld (foreign binaries)
  NIX_LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    stdenv.cc.cc.lib
    zlib
    libffi
    libxml2
    libusb1
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";

  shellHook = ''
    export PATH="$HOME/.cargo/bin:$PATH"

    # Erstelle libxml2.so.2 compat symlink (ESP-IDF clang braucht alte Version)
    mkdir -p "$PWD/.lib-compat"
    ln -sf "${pkgs.libxml2.out}/lib/libxml2.so" "$PWD/.lib-compat/libxml2.so.2"

    export LD_LIBRARY_PATH="$PWD/.lib-compat:${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libxml2.out}/lib:${pkgs.zlib}/lib:${pkgs.libffi}/lib:${pkgs.libusb1}/lib:$LD_LIBRARY_PATH"
    export NIX_LD_LIBRARY_PATH="$PWD/.lib-compat:${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libxml2.out}/lib:${pkgs.zlib}/lib:${pkgs.libffi}/lib:${pkgs.libusb1}/lib:$NIX_LD_LIBRARY_PATH"
    export BINDGEN_EXTRA_CLANG_ARGS="-fsigned-char"

    # Lade ESP-Toolchain falls vorhanden
    if [ -f "$HOME/export-esp.sh" ]; then
      source "$HOME/export-esp.sh"
      # Setze ESP-IDF Python-Umgebung um Nixpkgs zu überschreiben
      if [ -d "$PWD/.embuild/espressif/python_env/idf5.2_py3.13_env" ]; then
        export IDF_PYTHON_ENV_PATH="$PWD/.embuild/espressif/python_env/idf5.2_py3.13_env"
        export PATH="$IDF_PYTHON_ENV_PATH/bin:$PATH"
      fi
      # Skip Python version checks (NixOS hat neuere Versionen die kompatibel sind)
      export IDF_PYTHON_CHECK_CONSTRAINTS=0
      echo "✅ ESP Toolchain geladen"
    else
      echo ""
      echo "⚠️  ESP Toolchain nicht gefunden!"
      echo ""
      echo "Einmalig ausführen:"
      echo "  espup install --export-file ~/export-esp.sh"
      echo ""
      echo "Dann Shell neu starten oder:"
      echo "  source ~/export-esp.sh"
      echo ""
    fi

    echo "🌱 ESP32 Garden Guardian Dev Shell"
    echo ""
  '';
}
