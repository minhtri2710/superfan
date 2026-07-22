use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use superfan_lib::fan_actuation::console_user::active_console_uid;
use superfan_lib::fan_actuation::lease::ActuationLease;
use superfan_lib::fan_actuation::migration::remove_legacy_helper;
use superfan_lib::fan_actuation::protocol::{handle_command, FanCommand};
use superfan_lib::fan_actuation::smc_adapter::SmcFanHardware;
use superfan_lib::fan_actuation::socket::try_accept_once;

const SOCKET_DIR: &str = "/var/run/superfan";
const SOCKET_PATH: &str = "/var/run/superfan/fan-actuation.sock";

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_signal(_: libc::c_int) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}

fn main() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("fan actuation daemon must run as root");
        std::process::exit(1);
    }

    if let Err(error) = run() {
        eprintln!("fan actuation daemon failed: {error}");
        std::process::exit(1);
    }
}

#[allow(function_casts_as_integer)]
fn run() -> Result<(), String> {
    unsafe {
        libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
        libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
    }

    remove_legacy_helper()?;

    let mut hardware = SmcFanHardware;
    let _ = handle_command(FanCommand::RestoreAll, &mut hardware);

    fs::create_dir_all(SOCKET_DIR).map_err(|error| error.to_string())?;
    fs::set_permissions(SOCKET_DIR, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;

    let socket_path = Path::new(SOCKET_PATH);
    if socket_path.exists() {
        fs::remove_file(socket_path).map_err(|error| error.to_string())?;
    }

    let listener = UnixListener::bind(socket_path).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666))
        .map_err(|error| error.to_string())?;

    let mut lease = ActuationLease::new(Duration::from_secs(5), Instant::now());
    while !SHUTDOWN.load(Ordering::Relaxed) {
        if lease.should_restore(Instant::now()) {
            let _ = handle_command(FanCommand::RestoreAll, &mut hardware);
            lease.restored_all(Instant::now());
        }

        let authorized_uid = match active_console_uid() {
            Ok(uid) => uid,
            Err(_) => {
                let _ = handle_command(FanCommand::RestoreAll, &mut hardware);
                lease.restored_all(Instant::now());
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        match try_accept_once(&listener, authorized_uid, &mut hardware) {
            Ok(Some(FanCommand::SetTarget { fan_id, .. })) => {
                lease.set_manual(fan_id, Instant::now())
            }
            Ok(Some(FanCommand::SystemAuto { fan_id })) => {
                lease.set_system_auto(fan_id, Instant::now())
            }
            Ok(Some(FanCommand::RestoreAll)) => lease.restored_all(Instant::now()),
            Ok(Some(FanCommand::Status | FanCommand::Heartbeat)) => lease.heartbeat(Instant::now()),
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                let _ = handle_command(FanCommand::RestoreAll, &mut hardware);
                lease.restored_all(Instant::now());
                eprintln!("fan actuation request failed: {error}");
            }
        }
    }

    let _ = handle_command(FanCommand::RestoreAll, &mut hardware);
    let _ = fs::remove_file(socket_path);
    Ok(())
}
