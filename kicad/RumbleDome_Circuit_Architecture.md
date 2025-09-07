# RumbleDome Circuit Architecture

## System-Level Signal Flow
```
12V Vehicle → Power → MCU → CAN Bus ← Vehicle ECU
     ↓         ↓      ↓
   Fusing   Regulation Display
     ↓         ↓      ↓
  Protection  3.3V   Status
              ↓      
          Analog → ADC → Control Logic → PWM → Solenoid Driver
              ↑                          ↓
        Pressure                    Pneumatic
        Sensors                     Control
```

## Functional Blocks

### Block 1: Power Input & Protection
**Function:** Convert 12V automotive to clean 5V and 3.3V rails
**Inputs:** 12V battery (9-16V range)  
**Outputs:** 5V/2A, 3.3V/1A
**Key Requirements:** Reverse polarity protection, overcurrent limiting, noise filtering

**Circuit Description:**
- Reverse polarity MOSFET (P-channel) as first stage
- Automotive fuse (10A) for overcurrent protection  
- TVS diode for voltage spikes/load dump protection
- Buck converter (12V→5V) with inductor filtering
- LDO regulator (5V→3.3V) for clean analog supply
- Bulk capacitors for transient response

### Block 2: Pressure Sensor Interface (3x)
**Function:** Convert 0.5-4.5V sensor signals to 0-3.3V ADC range
**Inputs:** 3x pressure sensors (manifold, upper dome, lower dome)
**Outputs:** 3x scaled analog signals to MCU ADC
**Key Requirements:** <1% accuracy, noise immunity, overvoltage protection

**Circuit Description:**
- Voltage divider (precision resistors) for 4.5V→3.3V scaling
- RC low-pass filter (100Hz cutoff) for noise rejection
- ESD protection diodes on sensor inputs
- Bypass capacitors for supply decoupling

### Block 3: Solenoid Driver
**Function:** PWM control of 4-port pneumatic valve
**Inputs:** 3.3V PWM signal from MCU
**Outputs:** 12V switched current to solenoid (2A peak)
**Key Requirements:** 30Hz PWM, flyback protection, current sensing

**Circuit Description:**  
- N-channel MOSFET (logic-level) for switching
- Gate driver for fast switching/reduced losses
- Flyback diode for inductive load protection
- Current sense resistor + amplifier for feedback
- RC snubber for EMI reduction

### Block 4: CAN Bus Interface  
**Function:** ISO 11898 CAN communication with vehicle ECU
**Inputs:** CAN-H/CAN-L differential pair from vehicle
**Outputs:** Digital CAN signals to MCU
**Key Requirements:** 500kbps, automotive EMC compliance, galvanic isolation

**Circuit Description:**
- CAN transceiver IC (automotive grade)
- Common-mode choke for EMI filtering
- ESD protection on bus lines
- 120Ω termination resistor (switchable)
- Optional isolation transformer for harsh environments

### Block 5: Display Interface
**Function:** SPI control of 1.8" TFT LCD
**Inputs:** SPI signals from MCU (CLK, MOSI, CS, DC, RST)
**Outputs:** Display control
**Key Requirements:** 3.3V logic, fast refresh, readable in sunlight

**Circuit Description:**
- Level shifters if display requires different voltage
- Bypass capacitors for display supply
- Backlight LED driver (current-controlled)
- ESD protection on connector pins

## Signal Flow Between Blocks

### Power Flow:
12V → Protection → Buck → 5V Rail → LDO → 3.3V Rail

### Data Flow:
Sensors → ADC → MCU → Processing → PWM → Solenoid
ECU → CAN → MCU → Display

### Control Loop:
1. Read pressure sensors (ADC)
2. Read ECU torque demand (CAN)  
3. Calculate boost target (control algorithm)
4. Output PWM to solenoid driver
5. Update display status

## Interface Specifications

### Power Requirements:
- **12V input:** 9-16V operating, 24V transient survival
- **5V rail:** 2A capacity for solenoid driver
- **3.3V rail:** 1A capacity for MCU + peripherals

### Signal Levels:
- **Sensor inputs:** 0.5-4.5V → scaled to 0-3.3V
- **PWM output:** 3.3V logic → 12V switched
- **CAN bus:** Differential ±2V on 2.5V common mode

## Design Notes

### Safety Considerations:
- All fault conditions must fail safe (solenoid off = no boost)
- Overcurrent/overvoltage protection on all power rails
- ESD protection on all external interfaces
- Thermal protection for high-current paths

### EMC Considerations:
- Switching regulator layout critical for noise
- CAN bus common-mode filtering essential
- PWM switching frequency coordination
- Proper grounding and shielding strategy

### Component Selection Priorities:
1. **Automotive qualification** (AEC-Q100/Q101)
2. **Extended temperature range** (-40°C to +125°C)
3. **Vibration/shock resistance**
4. **Long-term availability** for production

## Physical Layout Considerations

### High-Priority Physical Separations

#### Power vs. Analog (CRITICAL)
- **Buck converter switching node** → Keep ≥10mm from sensor inputs
- **Power MOSFET/inductor** → Separate ground planes if possible
- **Switching frequency harmonics** → Will couple into pressure sensor ADC readings
- **Solution:** Dedicated analog ground plane, star grounding at one point

#### High-Current vs. Low-Current
- **12V solenoid switching** → Keep traces short, thick copper
- **PWM MOSFET** → Needs good thermal relief, keep from temperature-sensitive components  
- **Current sense resistor** → Shield from switching noise
- **Gate drive traces** → Keep short to minimize ringing/EMI

#### Digital Switching vs. Analog
- **MCU clock/SPI signals** → Route away from sensor analog paths
- **Display backlight PWM** → Can couple into audio-frequency sensor signals
- **CAN differential pairs** → Keep matched length, avoid crossing power planes

### Specific Layout Guidelines

#### Power Supply Block:
- **Buck converter:** Tight layout - input cap, switching node, inductor, output cap
- **Keep switching loop area minimal** (primary EMI source)
- **LDO regulator:** Place close to analog loads (sensors, ADC)
- **Heat sinks:** MOSFET and LDO may need thermal relief

#### Sensor Interface Block:
- **Voltage dividers:** Use precision resistors, keep traces short
- **RC filters:** Place close to ADC inputs
- **Shield sensor inputs** from digital switching if possible
- **Bypass caps:** One per sensor, close to connector

#### CAN Interface:
- **Differential pair routing:** Matched impedance (~120Ω), matched length
- **Common-mode choke:** Place close to connector
- **Keep CAN lines away from switching power**
- **Termination:** Switchable 120Ω resistor near connector

### Thermal Considerations

#### Heat Sources:
- **Buck converter MOSFET/inductor** (~1W dissipation)
- **Solenoid driver MOSFET** (~2W at 30Hz PWM)
- **LDO regulator** (~0.5W if poorly loaded)

#### Heat-Sensitive Components:
- **Precision resistors** (voltage dividers) - accuracy drifts with temperature
- **MCU** - keep reasonable operating temperature for reliability

### Grounding Strategy

**Recommended approach:**
- **Single-point star ground** for analog circuits
- **Separate digital/power ground planes** connected at one point
- **Shield ground** for CAN/external interfaces
- **Chassis ground** connection for automotive EMC compliance

### Connector Placement

- **Power input:** Heavy traces, fusing close to entry point
- **CAN bus:** Twisted pair, ferrite bead, ESD protection at connector
- **Sensors:** Shielded cables if long runs, ESD protection
- **Display:** Keep SPI traces short or use series termination

### Board Stack-up Considerations

**2-layer minimum, 4-layer preferred:**
- **Layer 1:** Components, signal routing
- **Layer 2:** Ground plane (analog section)  
- **Layer 3:** Power plane (digital section) - if 4-layer
- **Layer 4:** Power/mixed routing - if 4-layer

## Implementation Priority

**Phase 1:** Power supply and MCU core
**Phase 2:** Sensor interfaces and basic control
**Phase 3:** CAN communication and solenoid driver
**Phase 4:** Display interface and system integration

Each block can be designed and tested independently before integration.