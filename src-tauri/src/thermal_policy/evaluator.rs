use super::contract::{
    FanPlan, FanTarget, ThermalPolicyMode, ThermalPolicySettings, ThermalRule, ThermalTarget,
    PERFORMANCE_RULE, QUIET_RULE,
};
use crate::hardware_telemetry::contract::{
    Availability, FanReading, HardwareTelemetrySnapshot, TemperatureReadings,
};
use std::collections::BTreeMap;

const MAX_SNAPSHOT_AGE_MS: u64 = 5_000;
const DECREASE_HYSTERESIS_CELSIUS: f64 = 2.0;
const MAX_DECREASE_RPM_PER_SECOND: f64 = 400.0;

#[derive(Debug, Clone, Copy)]
struct PreviousTarget {
    rpm: i32,
    temperature_celsius: f64,
    evaluated_at_unix_ms: u64,
}

#[derive(Default)]
pub struct ThermalPolicyEvaluator {
    previous_targets: BTreeMap<usize, PreviousTarget>,
}

impl ThermalPolicyEvaluator {
    pub fn evaluate(
        &mut self,
        settings: &ThermalPolicySettings,
        snapshot: &HardwareTelemetrySnapshot,
        now_unix_ms: u64,
    ) -> FanPlan {
        if settings.mode == ThermalPolicyMode::SystemAuto
            || now_unix_ms.saturating_sub(snapshot.captured_at_unix_ms) > MAX_SNAPSHOT_AGE_MS
        {
            self.previous_targets.clear();
            return FanPlan::SystemAuto;
        }

        let temperatures = match &snapshot.temperatures {
            Availability::Available { value } => value,
            _ => return self.system_auto(),
        };
        let fans = match &snapshot.fans {
            Availability::Available { value } if !value.is_empty() => value,
            _ => return self.system_auto(),
        };
        let rules = rules_for(settings);
        let requested_percent = rules
            .iter()
            .filter(|rule| rule.active)
            .filter_map(|rule| {
                target_temperature(&rule.target, temperatures)
                    .map(|temperature| (temperature, interpolate_percent(rule, temperature)))
            })
            .max_by(|left, right| left.1.total_cmp(&right.1));
        let Some((temperature_celsius, fan_percent)) = requested_percent else {
            return self.system_auto();
        };

        let mut targets = Vec::with_capacity(fans.len());
        for fan in fans {
            let Some(requested_rpm) = percent_to_rpm(fan, fan_percent) else {
                return self.system_auto();
            };
            let rpm =
                self.apply_decrease_policy(fan.id, requested_rpm, temperature_celsius, now_unix_ms);
            targets.push(FanTarget {
                fan_id: fan.id,
                rpm,
            });
        }

        FanPlan::Targets { targets }
    }

    fn apply_decrease_policy(
        &mut self,
        fan_id: usize,
        requested_rpm: i32,
        temperature_celsius: f64,
        now_unix_ms: u64,
    ) -> i32 {
        if let Some(previous) = self.previous_targets.get(&fan_id).copied() {
            if requested_rpm < previous.rpm
                && temperature_celsius > previous.temperature_celsius - DECREASE_HYSTERESIS_CELSIUS
            {
                return previous.rpm;
            }
        }

        let rpm = match self.previous_targets.get(&fan_id).copied() {
            Some(previous) if requested_rpm < previous.rpm => {
                let elapsed_seconds =
                    now_unix_ms.saturating_sub(previous.evaluated_at_unix_ms) as f64 / 1_000.0;
                let maximum_decrease =
                    (MAX_DECREASE_RPM_PER_SECOND * elapsed_seconds).round() as i32;
                requested_rpm.max(previous.rpm - maximum_decrease)
            }
            _ => requested_rpm,
        };

        self.previous_targets.insert(
            fan_id,
            PreviousTarget {
                rpm,
                temperature_celsius,
                evaluated_at_unix_ms: now_unix_ms,
            },
        );
        rpm
    }

    fn system_auto(&mut self) -> FanPlan {
        self.previous_targets.clear();
        FanPlan::SystemAuto
    }
}

fn rules_for(settings: &ThermalPolicySettings) -> Vec<ThermalRule> {
    match settings.mode {
        ThermalPolicyMode::SystemAuto => vec![],
        ThermalPolicyMode::Quiet => vec![ThermalRule {
            id: "quiet".into(),
            name: "Quiet".into(),
            ..QUIET_RULE
        }],
        ThermalPolicyMode::Performance => vec![ThermalRule {
            id: "performance".into(),
            name: "Performance".into(),
            ..PERFORMANCE_RULE
        }],
        ThermalPolicyMode::Custom => settings.rules.clone(),
    }
}

fn target_temperature(target: &ThermalTarget, temperatures: &TemperatureReadings) -> Option<f64> {
    match target {
        ThermalTarget::Hottest => temperatures
            .cpu_celsius
            .into_iter()
            .chain(temperatures.gpu_celsius)
            .chain(temperatures.sensors.iter().map(|sensor| sensor.celsius))
            .max_by(|left, right| left.total_cmp(right)),
        ThermalTarget::Cpu => temperatures.cpu_celsius,
        ThermalTarget::Gpu => temperatures.gpu_celsius,
        ThermalTarget::SensorKey { key } => temperatures
            .sensors
            .iter()
            .find(|sensor| sensor.key == *key)
            .map(|sensor| sensor.celsius),
    }
}

fn interpolate_percent(rule: &ThermalRule, temperature_celsius: f64) -> f64 {
    if temperature_celsius <= rule.low_celsius {
        return f64::from(rule.min_fan_percent);
    }
    if temperature_celsius >= rule.high_celsius {
        return f64::from(rule.max_fan_percent);
    }

    let progress =
        (temperature_celsius - rule.low_celsius) / (rule.high_celsius - rule.low_celsius);
    f64::from(rule.min_fan_percent)
        + progress * f64::from(rule.max_fan_percent - rule.min_fan_percent)
}

fn percent_to_rpm(fan: &FanReading, percent: f64) -> Option<i32> {
    let minimum = fan.min_speed_rpm?;
    let maximum = fan.max_speed_rpm?;
    if maximum <= minimum {
        return None;
    }

    Some((f64::from(minimum) + f64::from(maximum - minimum) * percent / 100.0).round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_telemetry::contract::{FanActuationStatus, FanMode, TemperatureReading};

    fn snapshot(
        cpu: Option<f64>,
        gpu: Option<f64>,
        sensor: Option<f64>,
        captured_at: u64,
    ) -> HardwareTelemetrySnapshot {
        HardwareTelemetrySnapshot {
            temperatures: Availability::Available {
                value: TemperatureReadings {
                    cpu_celsius: cpu,
                    gpu_celsius: gpu,
                    sensors: sensor
                        .map(|celsius| TemperatureReading {
                            key: "Tp01".into(),
                            label: "P-Core 1".into(),
                            celsius,
                        })
                        .into_iter()
                        .collect(),
                },
            },
            fans: Availability::Available {
                value: vec![FanReading {
                    id: 0,
                    label: "Fan 1".into(),
                    speed_rpm: 2000,
                    min_speed_rpm: Some(1000),
                    max_speed_rpm: Some(5000),
                    target_speed_rpm: None,
                    mode: Some(FanMode::SystemAuto),
                }],
            },
            battery: Availability::NotPresent,
            fan_actuation_status: FanActuationStatus::Ready,
            captured_at_unix_ms: captured_at,
        }
    }

    fn rule(id: &str, target: ThermalTarget, low: f64, high: f64, min: u8, max: u8) -> ThermalRule {
        ThermalRule {
            id: id.into(),
            name: id.into(),
            target,
            low_celsius: low,
            high_celsius: high,
            min_fan_percent: min,
            max_fan_percent: max,
            active: true,
        }
    }

    #[test]
    fn system_auto_mode_produces_system_auto_plan() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        assert_eq!(
            evaluator.evaluate(
                &ThermalPolicySettings::default(),
                &snapshot(Some(70.0), None, None, 1_000),
                1_000
            ),
            FanPlan::SystemAuto
        );
    }

    #[test]
    fn stale_or_unavailable_snapshot_produces_system_auto_plan() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            ..Default::default()
        };
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(70.0), None, None, 1_000), 6_001),
            FanPlan::SystemAuto
        );
        let mut unavailable = snapshot(Some(70.0), None, None, 6_001);
        unavailable.temperatures = Availability::Unavailable {
            reason: "SMC access denied".into(),
        };
        assert_eq!(
            evaluator.evaluate(&settings, &unavailable, 6_001),
            FanPlan::SystemAuto
        );
    }

    #[test]
    fn interpolates_percent_into_each_fan_rpm_envelope() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("cpu", ThermalTarget::Cpu, 40.0, 80.0, 20, 100)],
        };
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(60.0), None, None, 1_000), 1_000),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 3400
                }]
            }
        );
    }

    #[test]
    fn hottest_target_uses_all_available_temperature_readings() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("hot", ThermalTarget::Hottest, 40.0, 80.0, 0, 100)],
        };
        assert_eq!(
            evaluator.evaluate(
                &settings,
                &snapshot(Some(55.0), Some(65.0), Some(75.0), 1_000),
                1_000
            ),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 4500
                }]
            }
        );
    }

    #[test]
    fn multiple_rules_choose_the_highest_target_for_each_fan() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![
                rule("cpu", ThermalTarget::Cpu, 40.0, 80.0, 0, 100),
                rule("gpu", ThermalTarget::Gpu, 40.0, 80.0, 50, 100),
            ],
        };
        assert_eq!(
            evaluator.evaluate(
                &settings,
                &snapshot(Some(60.0), Some(60.0), None, 1_000),
                1_000
            ),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 4000
                }]
            }
        );
    }

    #[test]
    fn missing_custom_target_cannot_produce_manual_actuation() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule(
                "missing",
                ThermalTarget::SensorKey {
                    key: "missing".into(),
                },
                40.0,
                80.0,
                0,
                100,
            )],
        };
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(60.0), None, None, 1_000), 1_000),
            FanPlan::SystemAuto
        );
    }

    #[test]
    fn increases_apply_immediately_but_decreases_require_hysteresis_and_rate_limit() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("cpu", ThermalTarget::Cpu, 40.0, 80.0, 0, 100)],
        };
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(80.0), None, None, 1_000), 1_000),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 5000
                }]
            }
        );
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(79.0), None, None, 2_000), 2_000),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 5000
                }]
            }
        );
        assert_eq!(
            evaluator.evaluate(&settings, &snapshot(Some(60.0), None, None, 3_000), 3_000),
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 4200
                }]
            }
        );
    }

    #[test]
    fn gradual_cooling_does_not_move_the_hysteresis_reference() {
        let mut evaluator = ThermalPolicyEvaluator::default();
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![rule("cpu", ThermalTarget::Cpu, 40.0, 80.0, 0, 100)],
        };

        let _ = evaluator.evaluate(&settings, &snapshot(Some(80.0), None, None, 1_000), 1_000);
        let blocked =
            evaluator.evaluate(&settings, &snapshot(Some(79.0), None, None, 2_000), 2_000);
        let decreased =
            evaluator.evaluate(&settings, &snapshot(Some(78.0), None, None, 3_000), 3_000);

        assert_eq!(
            blocked,
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 5000
                }]
            }
        );
        assert_eq!(
            decreased,
            FanPlan::Targets {
                targets: vec![FanTarget {
                    fan_id: 0,
                    rpm: 4800
                }]
            }
        );
    }
}
