pub mod adapters;
pub mod contract;
pub mod snapshot;

use adapters::{iokit::IokitAdapter, smc::SmcAdapter};
use contract::{FanActuationStatus, HardwareTelemetrySnapshot};
use snapshot::HardwareTelemetry;

pub fn capture(
    fan_actuation_status: FanActuationStatus,
    captured_at_unix_ms: u64,
) -> HardwareTelemetrySnapshot {
    HardwareTelemetry::new(SmcAdapter, IokitAdapter)
        .capture(fan_actuation_status, captured_at_unix_ms)
}
