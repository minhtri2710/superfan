use serde::Serialize;
use std::ffi::{c_char, c_int};

const ERROR_BUFFER_SIZE: usize = 512;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    NotRegistered,
    Enabled,
    RequiresApproval,
    NotFound,
    Unknown,
}

extern "C" {
    fn superfan_fan_daemon_status() -> c_int;
    fn superfan_register_fan_daemon(error_buffer: *mut c_char, error_buffer_length: usize) -> bool;
    fn superfan_open_login_items_settings();
}

pub fn status() -> ServiceStatus {
    match unsafe { superfan_fan_daemon_status() } {
        0 => ServiceStatus::NotRegistered,
        1 => ServiceStatus::Enabled,
        2 => ServiceStatus::RequiresApproval,
        3 => ServiceStatus::NotFound,
        _ => ServiceStatus::Unknown,
    }
}

pub fn register() -> Result<ServiceStatus, String> {
    let mut error_buffer = [0_i8; ERROR_BUFFER_SIZE];
    let registered =
        unsafe { superfan_register_fan_daemon(error_buffer.as_mut_ptr(), error_buffer.len()) };

    if registered {
        Ok(status())
    } else {
        let end = error_buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(error_buffer.len());
        let bytes = error_buffer[..end]
            .iter()
            .map(|value| *value as u8)
            .collect::<Vec<_>>();
        let message = String::from_utf8_lossy(&bytes).into_owned();
        Err(if message.is_empty() {
            "macOS could not register the Fan actuation service".into()
        } else {
            message
        })
    }
}

pub fn open_system_settings() {
    unsafe { superfan_open_login_items_settings() }
}

#[cfg(test)]
mod tests {
    use super::ServiceStatus;

    #[test]
    fn service_status_serializes_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&ServiceStatus::RequiresApproval).unwrap(),
            "\"requires_approval\""
        );
    }
}
