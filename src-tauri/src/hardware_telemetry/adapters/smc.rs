use crate::hardware_telemetry::contract::{
    Availability, FanMode, FanReading, TemperatureReading, TemperatureReadings,
};
use crate::hardware_telemetry::snapshot::SmcTelemetryAdapter;
use crate::smc;

#[derive(Clone, Copy, Default)]
pub struct SmcAdapter;

impl SmcTelemetryAdapter for SmcAdapter {
    fn temperatures(&self) -> Availability<TemperatureReadings> {
        if !smc::ensure_smc_open() {
            return Availability::Unavailable {
                reason: "SMC access is unavailable".into(),
            };
        }

        Availability::Available {
            value: TemperatureReadings {
                cpu_celsius: smc::get_cpu_temperature(),
                gpu_celsius: smc::get_gpu_temperature(),
                sensors: smc::get_all_sensors()
                    .into_iter()
                    .map(|(key, label, celsius)| TemperatureReading {
                        key,
                        label,
                        celsius,
                    })
                    .collect(),
            },
        }
    }

    fn fans(&self) -> Availability<Vec<FanReading>> {
        if !smc::ensure_smc_open() {
            return Availability::Unavailable {
                reason: "SMC access is unavailable".into(),
            };
        }

        let fans = smc::get_fan_readings()
            .into_iter()
            .map(|fan| FanReading {
                id: fan.id,
                label: fan.label,
                speed_rpm: fan.speed_rpm,
                min_speed_rpm: fan.min_speed_rpm,
                max_speed_rpm: fan.max_speed_rpm,
                target_speed_rpm: fan.target_speed_rpm,
                mode: fan.is_manual.map(|is_manual| {
                    if is_manual {
                        FanMode::Manual
                    } else {
                        FanMode::SystemAuto
                    }
                }),
            })
            .collect::<Vec<_>>();

        if fans.is_empty() {
            Availability::NotPresent
        } else {
            Availability::Available { value: fans }
        }
    }
}
