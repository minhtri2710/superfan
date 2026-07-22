use super::contract::{
    Availability, BatteryReading, FanActuationStatus, FanReading, HardwareTelemetrySnapshot,
    TemperatureReadings,
};

pub trait SmcTelemetryAdapter {
    fn temperatures(&self) -> Availability<TemperatureReadings>;
    fn fans(&self) -> Availability<Vec<FanReading>>;
}

pub trait IokitTelemetryAdapter {
    fn battery(&self) -> Availability<BatteryReading>;
}

pub struct HardwareTelemetry<S, I> {
    smc: S,
    iokit: I,
}

impl<S, I> HardwareTelemetry<S, I>
where
    S: SmcTelemetryAdapter,
    I: IokitTelemetryAdapter,
{
    pub fn new(smc: S, iokit: I) -> Self {
        Self { smc, iokit }
    }

    pub fn capture(
        &self,
        fan_actuation_status: FanActuationStatus,
        captured_at_unix_ms: u64,
    ) -> HardwareTelemetrySnapshot {
        HardwareTelemetrySnapshot {
            temperatures: self.smc.temperatures(),
            fans: self.smc.fans(),
            battery: self.iokit.battery(),
            fan_actuation_status,
            captured_at_unix_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_telemetry::adapters::fixture::FixtureTelemetryAdapter;
    use crate::hardware_telemetry::contract::{FanMode, TemperatureReadings};

    #[test]
    fn fixture_adapter_produces_a_complete_snapshot() {
        let fixture = FixtureTelemetryAdapter {
            temperatures: Availability::Available {
                value: TemperatureReadings {
                    cpu_celsius: Some(70.0),
                    gpu_celsius: None,
                    sensors: vec![],
                },
            },
            fans: Availability::Available {
                value: vec![FanReading {
                    id: 0,
                    label: "Fan 1".into(),
                    speed_rpm: 2200,
                    min_speed_rpm: Some(1200),
                    max_speed_rpm: Some(6000),
                    target_speed_rpm: None,
                    mode: Some(FanMode::SystemAuto),
                }],
            },
            battery: Availability::Available {
                value: BatteryReading {
                    charge_percent: Some(80),
                    temperature_celsius: None,
                    is_charging: Some(true),
                    cycle_count: None,
                    power_watts: None,
                },
            },
        };

        let snapshot = HardwareTelemetry::new(fixture.clone(), fixture)
            .capture(FanActuationStatus::Ready, 1_700_000_000_000);

        assert_eq!(snapshot.captured_at_unix_ms, 1_700_000_000_000);
        assert_eq!(snapshot.fan_actuation_status, FanActuationStatus::Ready);
        assert!(matches!(
            snapshot.temperatures,
            Availability::Available { .. }
        ));
        assert!(matches!(snapshot.fans, Availability::Available { .. }));
        assert!(matches!(snapshot.battery, Availability::Available { .. }));
    }

    #[test]
    fn snapshot_preserves_adapter_unavailability_without_fallbacks() {
        let fixture = FixtureTelemetryAdapter {
            temperatures: Availability::Unavailable {
                reason: "SMC access denied".into(),
            },
            fans: Availability::Unavailable {
                reason: "SMC access denied".into(),
            },
            battery: Availability::NotPresent,
        };

        let snapshot = HardwareTelemetry::new(fixture.clone(), fixture)
            .capture(FanActuationStatus::Unavailable, 1_700_000_000_000);

        assert_eq!(
            snapshot.temperatures,
            Availability::Unavailable {
                reason: "SMC access denied".into()
            }
        );
        assert_eq!(snapshot.battery, Availability::NotPresent);
    }
}
