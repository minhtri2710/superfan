use crate::hardware_telemetry::contract::{Availability, BatteryReading};
use crate::hardware_telemetry::snapshot::IokitTelemetryAdapter;
use crate::smc;

#[derive(Clone, Copy, Default)]
pub struct IokitAdapter;

impl IokitTelemetryAdapter for IokitAdapter {
    fn battery(&self) -> Availability<BatteryReading> {
        match smc::get_battery_reading() {
            Some(battery) => Availability::Available {
                value: BatteryReading {
                    charge_percent: battery.charge_percent,
                    temperature_celsius: battery.temperature_celsius,
                    is_charging: battery.is_charging,
                    cycle_count: battery.cycle_count,
                    power_watts: battery.power_watts,
                },
            },
            None => Availability::NotPresent,
        }
    }
}
