use crate::hardware_telemetry::contract::{
    Availability, BatteryReading, FanReading, TemperatureReadings,
};
use crate::hardware_telemetry::snapshot::{IokitTelemetryAdapter, SmcTelemetryAdapter};

#[derive(Clone)]
pub struct FixtureTelemetryAdapter {
    pub temperatures: Availability<TemperatureReadings>,
    pub fans: Availability<Vec<FanReading>>,
    pub battery: Availability<BatteryReading>,
}

impl SmcTelemetryAdapter for FixtureTelemetryAdapter {
    fn temperatures(&self) -> Availability<TemperatureReadings> {
        self.temperatures.clone()
    }

    fn fans(&self) -> Availability<Vec<FanReading>> {
        self.fans.clone()
    }
}

impl IokitTelemetryAdapter for FixtureTelemetryAdapter {
    fn battery(&self) -> Availability<BatteryReading> {
        self.battery.clone()
    }
}
