use super::bootstrap::{self, ServiceStatus};
use super::protocol::{FanCommand, FanResponse};
use super::socket::send_command;
use serde::Serialize;
use std::path::Path;

pub const SOCKET_PATH: &str = "/var/run/superfan/fan-actuation.sock";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActuationStatus {
    NotRegistered,
    RequiresApproval,
    Ready,
    Unavailable,
}

pub fn status() -> ActuationStatus {
    match bootstrap::status() {
        ServiceStatus::NotRegistered | ServiceStatus::NotFound | ServiceStatus::Unknown => {
            ActuationStatus::NotRegistered
        }
        ServiceStatus::RequiresApproval => ActuationStatus::RequiresApproval,
        ServiceStatus::Enabled => match command(FanCommand::Status) {
            Ok(FanResponse::Ready { .. }) => ActuationStatus::Ready,
            _ => ActuationStatus::Unavailable,
        },
    }
}

pub fn command(command: FanCommand) -> Result<FanResponse, String> {
    let response = send_command(Path::new(SOCKET_PATH), &command)?;
    match response {
        FanResponse::Error { message } => Err(message),
        response => Ok(response),
    }
}

pub fn set_target(fan_id: usize, rpm: i32) -> Result<(), String> {
    expect_ok(command(FanCommand::SetTarget { fan_id, rpm })?)
}

pub fn system_auto(fan_id: usize) -> Result<(), String> {
    expect_ok(command(FanCommand::SystemAuto { fan_id })?)
}

pub fn restore_all() -> Result<(), String> {
    expect_ok(command(FanCommand::RestoreAll)?)
}

pub fn heartbeat() -> Result<(), String> {
    expect_ok(command(FanCommand::Heartbeat)?)
}

fn expect_ok(response: FanResponse) -> Result<(), String> {
    if response == FanResponse::Ok {
        Ok(())
    } else {
        Err("unexpected response from Fan actuation service".into())
    }
}

#[cfg(test)]
mod tests {
    use super::ActuationStatus;

    #[test]
    fn actuation_status_serializes_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&ActuationStatus::RequiresApproval).unwrap(),
            "\"requires_approval\""
        );
    }
}
