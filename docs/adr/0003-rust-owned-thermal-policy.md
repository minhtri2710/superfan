---
status: accepted
---

# Evaluate Thermal policy in Rust

SuperFan will evaluate Thermal policy in the Rust backend from each Hardware telemetry snapshot and produce a Fan plan. System Auto and stale, missing, or unavailable hardware data produce a System Auto Fan plan; active rules use explicit Celsius and fan-percent ranges, the highest requested target wins, increases apply immediately, and decreases use 2°C hysteresis with a 400 RPM-per-second limit. Persisted settings and Fan actuation remain outside the pure evaluator interface. This adds backend state but gives the policy module locality, deterministic fixture tests, and one writer for automatic fan behavior.
