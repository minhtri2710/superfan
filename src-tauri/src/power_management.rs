use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use crate::thermal_policy::state::{FanActuation, SettingsStore, ThermalPolicyState};

static IS_SLEEPING: AtomicBool = AtomicBool::new(false);
static mut RESTORE_FN: Option<Box<dyn Fn() -> Result<(), String> + Send + Sync>> = None;

pub(crate) fn is_sleeping() -> bool {
    IS_SLEEPING.load(Ordering::Relaxed)
}

#[cfg(test)]
pub(crate) fn set_sleeping(sleeping: bool) {
    IS_SLEEPING.store(sleeping, Ordering::Relaxed);
}

extern "C" fn on_will_sleep() {
    IS_SLEEPING.store(true, Ordering::Relaxed);
    unsafe {
        if let Some(ref restore) = RESTORE_FN {
            let _ = restore();
        }
    }
}

extern "C" fn on_did_wake() {
    IS_SLEEPING.store(false, Ordering::Relaxed);
}

#[cfg(target_os = "macos")]
extern "C" {
    fn superfan_register_sleep_wake_callbacks(
        will_sleep: extern "C" fn(),
        did_wake: extern "C" fn(),
    );
}

pub(crate) fn register_power_events<S, F>(policy_state: Arc<Mutex<ThermalPolicyState<S, F>>>)
where
    S: SettingsStore + Send + 'static,
    F: FanActuation + Send + 'static,
{
    let state_clone = policy_state;
    let restore = Box::new(move || -> Result<(), String> {
        let mut state = state_clone.lock().map_err(|e| e.to_string())?;
        state.force_restore_system_auto()
    });

    unsafe {
        RESTORE_FN = Some(restore);
    }

    #[cfg(target_os = "macos")]
    {
        unsafe {
            superfan_register_sleep_wake_callbacks(on_will_sleep, on_did_wake);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_state_toggles_correctly() {
        set_sleeping(false);
        assert!(!is_sleeping());

        on_will_sleep();
        assert!(is_sleeping());

        on_did_wake();
        assert!(!is_sleeping());
    }
}
