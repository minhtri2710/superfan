use super::protocol::{handle_command, FanCommand, FanHardware, FanResponse};
use std::io::{BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;

const MAX_REQUEST_BYTES: u64 = 4096;
const IO_TIMEOUT: Duration = Duration::from_secs(2);

pub fn send_command(path: &Path, command: &FanCommand) -> Result<FanResponse, String> {
    let mut stream = UnixStream::connect(path).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(IO_TIMEOUT))
        .map_err(|error| error.to_string())?;

    let mut payload = serde_json::to_vec(command).map_err(|error| error.to_string())?;
    payload.push(b'\n');
    stream
        .write_all(&payload)
        .map_err(|error| error.to_string())?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| error.to_string())?;

    let response = read_message(&stream)?;
    serde_json::from_slice(&response).map_err(|error| error.to_string())
}

pub fn serve_connection(
    mut stream: UnixStream,
    authorized_uid: u32,
    hardware: &mut impl FanHardware,
) -> Result<Option<FanCommand>, String> {
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(IO_TIMEOUT))
        .map_err(|error| error.to_string())?;

    let peer_uid = peer_uid(&stream).map_err(|error| error.to_string())?;
    if peer_uid != authorized_uid {
        write_response(
            &mut stream,
            &FanResponse::Error {
                message: "fan actuation is limited to the active console user".into(),
            },
        )?;
        return Ok(None);
    }

    let command = match read_message(&stream).and_then(|payload| {
        serde_json::from_slice::<FanCommand>(&payload).map_err(|e| e.to_string())
    }) {
        Ok(command) => command,
        Err(message) => {
            let _ = handle_command(FanCommand::RestoreAll, hardware);
            write_response(&mut stream, &FanResponse::Error { message })?;
            return Ok(None);
        }
    };

    let response = handle_command(command.clone(), hardware);
    if let Err(error) = write_response(&mut stream, &response) {
        let _ = handle_command(FanCommand::RestoreAll, hardware);
        return Err(error);
    }
    Ok(Some(command))
}

pub fn accept_once(
    listener: &UnixListener,
    authorized_uid: u32,
    hardware: &mut impl FanHardware,
) -> Result<Option<FanCommand>, String> {
    let (stream, _) = listener.accept().map_err(|error| error.to_string())?;
    serve_connection(stream, authorized_uid, hardware)
}

pub fn try_accept_once(
    listener: &UnixListener,
    authorized_uid: u32,
    hardware: &mut impl FanHardware,
) -> Result<Option<FanCommand>, String> {
    match listener.accept() {
        Ok((stream, _)) => serve_connection(stream, authorized_uid, hardware),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn read_message(stream: &UnixStream) -> Result<Vec<u8>, String> {
    let mut message = Vec::new();
    BufReader::new(stream)
        .take(MAX_REQUEST_BYTES + 1)
        .read_to_end(&mut message)
        .map_err(|error| error.to_string())?;

    if message.len() as u64 > MAX_REQUEST_BYTES {
        return Err("fan actuation request is too large".into());
    }
    if message.last() != Some(&b'\n') {
        return Err("fan actuation request must end with a newline".into());
    }
    message.pop();
    Ok(message)
}

fn write_response(stream: &mut UnixStream, response: &FanResponse) -> Result<(), String> {
    let mut payload = serde_json::to_vec(response).map_err(|error| error.to_string())?;
    payload.push(b'\n');
    stream
        .write_all(&payload)
        .map_err(|error| error.to_string())
}

fn peer_uid(stream: &UnixStream) -> std::io::Result<u32> {
    let mut uid = 0;
    let mut gid = 0;
    // Apple documents getpeereid(3) as a reliable credential check for connected
    // SOCK_STREAM Unix-domain peers.
    // Source: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html
    let result = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) };
    if result == 0 {
        Ok(uid)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    use super::{accept_once, send_command};
    use crate::fan_actuation::protocol::{FanCommand, FanEnvelope, FanHardware, FanResponse};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn socket_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "superfan-{name}-{}-{nonce}.sock",
            std::process::id()
        ))
    }

    #[test]
    fn accepts_commands_from_authorized_peer() {
        let path = socket_path("authorized");
        let listener = UnixListener::bind(&path).unwrap();
        let uid = unsafe { libc::geteuid() };
        let server = thread::spawn(move || {
            let mut hardware = FixtureHardware::default().with_fan(0, 1200, 6000);
            let _ = accept_once(&listener, uid, &mut hardware).unwrap();
            hardware
        });

        let response = send_command(
            &path,
            &FanCommand::SetTarget {
                fan_id: 0,
                rpm: 3200,
            },
        )
        .unwrap();

        let hardware = server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(response, FanResponse::Ok);
        assert_eq!(hardware.targets.get(&0), Some(&3200));
    }

    #[test]
    fn rejects_commands_from_other_users() {
        let path = socket_path("unauthorized");
        let listener = UnixListener::bind(&path).unwrap();
        let uid = unsafe { libc::geteuid() };
        let server = thread::spawn(move || {
            let mut hardware = FixtureHardware::default().with_fan(0, 1200, 6000);
            let _ = accept_once(&listener, uid.saturating_add(1), &mut hardware).unwrap();
            hardware
        });

        let response = send_command(&path, &FanCommand::Status).unwrap();

        let hardware = server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(matches!(response, FanResponse::Error { .. }));
        assert!(hardware.targets.is_empty());
        assert!(hardware.restored.is_empty());
    }

    #[test]
    fn malformed_authorized_request_restores_system_auto() {
        let path = socket_path("malformed");
        let listener = UnixListener::bind(&path).unwrap();
        let uid = unsafe { libc::geteuid() };
        let server = thread::spawn(move || {
            let mut hardware = FixtureHardware::default()
                .with_fan(0, 1200, 6000)
                .with_fan(1, 1200, 6000);
            let _ = accept_once(&listener, uid, &mut hardware).unwrap();
            hardware
        });

        let mut stream = UnixStream::connect(&path).unwrap();
        stream.write_all(b"{not-json}\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        let hardware = server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(response.contains("error"));
        assert_eq!(hardware.restored, vec![0, 1]);
    }
}
