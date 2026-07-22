use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum FanCommand {
    Status,
    Heartbeat,
    SetTarget { fan_id: usize, rpm: i32 },
    SystemAuto { fan_id: usize },
    RestoreAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanEnvelope {
    pub id: usize,
    pub min_rpm: i32,
    pub max_rpm: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FanResponse {
    Ok,
    Ready { fans: Vec<FanEnvelope> },
    Error { message: String },
}

pub trait FanHardware {
    fn envelopes(&self) -> Result<Vec<FanEnvelope>, String>;
    fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String>;
    fn system_auto(&mut self, fan_id: usize) -> Result<(), String>;
}

pub fn handle_command(command: FanCommand, hardware: &mut impl FanHardware) -> FanResponse {
    match command {
        FanCommand::Status => match hardware.envelopes() {
            Ok(fans) => FanResponse::Ready { fans },
            Err(message) => FanResponse::Error { message },
        },
        FanCommand::Heartbeat => FanResponse::Ok,
        FanCommand::SetTarget { fan_id, rpm } => {
            let fans = match hardware.envelopes() {
                Ok(fans) => fans,
                Err(message) => return FanResponse::Error { message },
            };
            let Some(fan) = fans.iter().find(|fan| fan.id == fan_id) else {
                return FanResponse::Error {
                    message: format!("fan {fan_id} is unavailable"),
                };
            };
            if rpm < fan.min_rpm || rpm > fan.max_rpm {
                return FanResponse::Error {
                    message: format!(
                        "target {rpm} RPM is outside fan {fan_id} range {}-{} RPM",
                        fan.min_rpm, fan.max_rpm
                    ),
                };
            }
            match hardware.set_target(fan_id, rpm) {
                Ok(()) => FanResponse::Ok,
                Err(message) => FanResponse::Error { message },
            }
        }
        FanCommand::SystemAuto { fan_id } => {
            let fans = match hardware.envelopes() {
                Ok(fans) => fans,
                Err(message) => return FanResponse::Error { message },
            };
            if !fans.iter().any(|fan| fan.id == fan_id) {
                return FanResponse::Error {
                    message: format!("fan {fan_id} is unavailable"),
                };
            }
            match hardware.system_auto(fan_id) {
                Ok(()) => FanResponse::Ok,
                Err(message) => FanResponse::Error { message },
            }
        }
        FanCommand::RestoreAll => {
            let fans = match hardware.envelopes() {
                Ok(fans) => fans,
                Err(message) => return FanResponse::Error { message },
            };
            for fan in fans {
                if let Err(message) = hardware.system_auto(fan.id) {
                    return FanResponse::Error { message };
                }
            }
            FanResponse::Ok
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_command, FanCommand, FanEnvelope, FanHardware, FanResponse};
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct FixtureHardware {
        fans: BTreeMap<usize, (i32, i32)>,
        targets: BTreeMap<usize, i32>,
        restored: Vec<usize>,
    }

    impl FixtureHardware {
        fn with_fan(mut self, id: usize, min_rpm: i32, max_rpm: i32) -> Self {
            self.fans.insert(id, (min_rpm, max_rpm));
            self
        }
    }

    impl FanHardware for FixtureHardware {
        fn envelopes(&self) -> Result<Vec<FanEnvelope>, String> {
            Ok(self
                .fans
                .iter()
                .map(|(id, (min_rpm, max_rpm))| FanEnvelope {
                    id: *id,
                    min_rpm: *min_rpm,
                    max_rpm: *max_rpm,
                })
                .collect())
        }

        fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String> {
            self.targets.insert(fan_id, rpm);
            Ok(())
        }

        fn system_auto(&mut self, fan_id: usize) -> Result<(), String> {
            self.restored.push(fan_id);
            Ok(())
        }
    }

    #[test]
    fn rejects_target_outside_hardware_envelope() {
        let mut hardware = FixtureHardware::default().with_fan(0, 1200, 6000);

        let response = handle_command(
            FanCommand::SetTarget {
                fan_id: 0,
                rpm: 6100,
            },
            &mut hardware,
        );

        assert!(matches!(response, FanResponse::Error { .. }));
        assert!(hardware.targets.is_empty());
    }

    #[test]
    fn applies_valid_target() {
        let mut hardware = FixtureHardware::default().with_fan(0, 1200, 6000);

        let response = handle_command(
            FanCommand::SetTarget {
                fan_id: 0,
                rpm: 3200,
            },
            &mut hardware,
        );

        assert_eq!(response, FanResponse::Ok);
        assert_eq!(hardware.targets.get(&0), Some(&3200));
    }

    #[test]
    fn rejects_system_auto_for_unknown_fan() {
        let mut hardware = FixtureHardware::default().with_fan(0, 1200, 6000);

        let response = handle_command(FanCommand::SystemAuto { fan_id: 99 }, &mut hardware);

        assert!(matches!(response, FanResponse::Error { .. }));
        assert!(hardware.restored.is_empty());
    }

    #[test]
    fn restore_all_returns_every_fan_to_system_auto() {
        let mut hardware = FixtureHardware::default()
            .with_fan(0, 1200, 6000)
            .with_fan(1, 1200, 6000);

        let response = handle_command(FanCommand::RestoreAll, &mut hardware);

        assert_eq!(response, FanResponse::Ok);
        assert_eq!(hardware.restored, vec![0, 1]);
    }
}
