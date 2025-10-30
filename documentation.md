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
  - **Temperature:** module **0x00**, function **0x04** ⇒ 32‑bit big‑endian, result / **65536**.
