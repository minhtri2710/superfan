# SuperFan

SuperFan observes Mac hardware thermals and manages fan behavior while preserving macOS as the fail-safe authority.

## Language

**Hardware telemetry snapshot**:
A point-in-time normalized view of temperatures, fans, battery state, and hardware access health.
_Avoid_: Telemetry data, sensor payload

**Thermal policy**:
The rules and presets that translate a Hardware telemetry snapshot into a Fan plan.
_Avoid_: Smart curve, fan rule engine

**Fan plan**:
The desired fan behavior produced by Thermal policy, including target speeds or a return to System Auto.
_Avoid_: Fan command, curve result

**Fan actuation**:
The controlled application of a Fan plan to the Mac's fans.
_Avoid_: Fan control, helper command

**System Auto**:
The fail-safe state in which macOS owns fan behavior.
_Avoid_: Auto mode, default mode
