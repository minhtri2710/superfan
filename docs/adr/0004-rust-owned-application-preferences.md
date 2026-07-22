---
status: accepted
---

# Use a Rust-owned Application preferences module

SuperFan will own display temperature unit, telemetry cadence, and launch-at-login state in a Rust Application preferences module. The module exposes one authoritative preferences snapshot and one tagged update interface; validation, Tauri-store persistence, macOS autostart reconciliation, and telemetry-loop cadence notification remain behind that interface. Store and autostart adapters provide hardware-independent tests, macOS remains authoritative for launch-at-login state, and Rust generates the TypeScript contracts. This adds backend state but gives settings changes locality, makes the interface the test surface, and prevents frontend controls from diverging from runtime behavior.
