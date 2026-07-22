# SuperFan

SuperFan observes Mac hardware thermals and manages fan behavior while preserving macOS as the fail-safe authority.

## Language

**Hardware telemetry snapshot**:
A point-in-time normalized view of temperatures, fans, battery state, and hardware access health. Fields name Celsius, RPM, percent, watts, and capture time explicitly; each hardware group reports `available`, `not_present`, or `unavailable` instead of display fallbacks.
_Avoid_: Telemetry data, sensor payload

**Thermal policy**:
The Rust-owned rules and presets that translate each Hardware telemetry snapshot into a Fan plan. It uses hottest-target-wins, treats data older than five seconds as stale, applies RPM increases immediately, and gates decreases with 2°C hysteresis and a 400 RPM-per-second limit.
_Avoid_: Smart curve, fan rule engine

**Fan plan**:
The desired fan behavior produced by Thermal policy: either per-fan RPM targets or a return to System Auto. It is the only output of policy evaluation and contains no socket or persistence behavior.
_Avoid_: Fan command, curve result

**Fan actuation**:
The controlled application of a Fan plan to the Mac's fans.
_Avoid_: Fan control, helper command

**System Auto**:
The fail-safe state in which macOS owns fan behavior.
_Avoid_: Auto mode, default mode
