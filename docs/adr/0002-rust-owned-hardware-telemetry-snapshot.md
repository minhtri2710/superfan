---
status: accepted
---

# Use a Rust-owned Hardware telemetry snapshot

SuperFan will expose one Rust-owned Hardware telemetry snapshot with unit-bearing fields and explicit `available`, `not_present`, or `unavailable` states. SMC, IOKit, and fixture adapters remain behind the telemetry module seam, TypeScript types are generated from the Rust contract, and fake display fallbacks are not permitted. This increases contract ceremony but improves locality, gives tests a hardware-independent interface, and prevents Thermal policy from acting on invented measurements.
