use std::collections::BTreeSet;
use std::time::{Duration, Instant};

pub struct ActuationLease {
    timeout: Duration,
    last_heartbeat: Instant,
    manual_fans: BTreeSet<usize>,
}

impl ActuationLease {
    pub fn new(timeout: Duration, now: Instant) -> Self {
        Self {
            timeout,
            last_heartbeat: now,
            manual_fans: BTreeSet::new(),
        }
    }

    pub fn heartbeat(&mut self, now: Instant) {
        self.last_heartbeat = now;
    }

    pub fn set_manual(&mut self, fan_id: usize, now: Instant) {
        self.manual_fans.insert(fan_id);
        self.heartbeat(now);
    }

    pub fn set_system_auto(&mut self, fan_id: usize, now: Instant) {
        self.manual_fans.remove(&fan_id);
        self.heartbeat(now);
    }

    pub fn restored_all(&mut self, now: Instant) {
        self.manual_fans.clear();
        self.heartbeat(now);
    }

    pub fn should_restore(&self, now: Instant) -> bool {
        !self.manual_fans.is_empty()
            && now.saturating_duration_since(self.last_heartbeat) >= self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::ActuationLease;
    use std::time::{Duration, Instant};

    #[test]
    fn expires_manual_control_without_heartbeat() {
        let started = Instant::now();
        let mut lease = ActuationLease::new(Duration::from_secs(5), started);
        lease.set_manual(0, started);

        assert!(lease.should_restore(started + Duration::from_secs(5)));
    }

    #[test]
    fn heartbeat_keeps_manual_control_active() {
        let started = Instant::now();
        let mut lease = ActuationLease::new(Duration::from_secs(5), started);
        lease.set_manual(0, started);
        lease.heartbeat(started + Duration::from_secs(4));

        assert!(!lease.should_restore(started + Duration::from_secs(8)));
    }

    #[test]
    fn system_auto_clears_the_manual_lease() {
        let started = Instant::now();
        let mut lease = ActuationLease::new(Duration::from_secs(5), started);
        lease.set_manual(0, started);
        lease.set_system_auto(0, started + Duration::from_secs(1));

        assert!(!lease.should_restore(started + Duration::from_secs(10)));
    }
}
