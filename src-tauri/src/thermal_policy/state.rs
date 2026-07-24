use super::contract::{FanPlan, ThermalPolicyMode, ThermalPolicySettings, ThermalRule};
use super::evaluator::ThermalPolicyEvaluator;
use super::settings;
use crate::hardware_telemetry::contract::{FanActuationStatus, HardwareTelemetrySnapshot};
use std::collections::BTreeMap;

pub(crate) trait SettingsStore {
    fn load(&self) -> Result<Option<ThermalPolicySettings>, String>;
    fn save(&mut self, settings: &ThermalPolicySettings) -> Result<(), String>;
}

pub(crate) trait FanActuation {
    fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String>;
    fn system_auto(&mut self, fan_id: usize) -> Result<(), String>;
    fn restore_all(&mut self) -> Result<(), String>;
    fn heartbeat(&mut self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectFanActuationRequest {
    Target { fan_id: usize, rpm: i32 },
    SystemAuto { fan_id: usize },
}

#[derive(Debug, Clone)]
pub enum ThermalPolicyChange {
    SelectMode(ThermalPolicyMode),
    UpsertRule(ThermalRule),
    DeleteRule(String),
}

pub(crate) struct ThermalPolicyState<S, F> {
    store: S,
    fan_actuation: F,
    current: ThermalPolicySettings,
    evaluator: ThermalPolicyEvaluator,
    applied_targets: BTreeMap<usize, i32>,
    manual_plan_active: bool,
    system_auto_confirmed: bool,
}

impl<S: SettingsStore, F: FanActuation> ThermalPolicyState<S, F> {
    pub(crate) fn load(store: S, fan_actuation: F) -> Self {
        let current = store
            .load()
            .ok()
            .flatten()
            .filter(|settings| settings::validate(settings).is_ok())
            .unwrap_or_default();
        Self {
            store,
            fan_actuation,
            current,
            evaluator: ThermalPolicyEvaluator::default(),
            applied_targets: BTreeMap::new(),
            manual_plan_active: false,
            system_auto_confirmed: false,
        }
    }

    pub(crate) fn current(&self) -> ThermalPolicySettings {
        self.current.clone()
    }

    pub(crate) fn direct_actuation(
        &mut self,
        request: DirectFanActuationRequest,
    ) -> Result<(), String> {
        if self.current.mode != ThermalPolicyMode::SystemAuto {
            return Err("direct Fan actuation is disabled while Thermal policy is active".into());
        }
        match request {
            DirectFanActuationRequest::Target { fan_id, rpm } => {
                self.fan_actuation.set_target(fan_id, rpm)
            }
            DirectFanActuationRequest::SystemAuto { fan_id } => {
                self.fan_actuation.system_auto(fan_id)
            }
        }
    }

    pub(crate) fn process_snapshot(
        &mut self,
        snapshot: &HardwareTelemetrySnapshot,
        now_unix_ms: u64,
    ) -> Result<FanPlan, String> {
        let plan = self
            .evaluator
            .evaluate(&self.current, snapshot, now_unix_ms);
        self.apply_plan(&plan)?;
        if self.current.mode == ThermalPolicyMode::SystemAuto
            && snapshot.fan_actuation_status == FanActuationStatus::Ready
        {
            if let Err(error) = self.fan_actuation.heartbeat() {
                self.recover_from_actuation_failure();
                return Err(error);
            }
        }
        Ok(plan)
    }

    fn apply_plan(&mut self, plan: &FanPlan) -> Result<(), String> {
        match plan {
            FanPlan::SystemAuto => self.restore_system_auto(),
            FanPlan::Targets { targets } => {
                for target in targets
                    .iter()
                    .filter(|target| self.applied_targets.get(&target.fan_id) != Some(&target.rpm))
                {
                    if let Err(error) = self.fan_actuation.set_target(target.fan_id, target.rpm) {
                        self.recover_from_actuation_failure();
                        return Err(error);
                    }
                }

                self.applied_targets = targets
                    .iter()
                    .map(|target| (target.fan_id, target.rpm))
                    .collect();
                self.manual_plan_active = true;
                self.system_auto_confirmed = false;
                if let Err(error) = self.fan_actuation.heartbeat() {
                    self.recover_from_actuation_failure();
                    return Err(error);
                }
                Ok(())
            }
        }
    }

    fn restore_system_auto(&mut self) -> Result<(), String> {
        if self.manual_plan_active || !self.system_auto_confirmed {
            return self.force_restore_system_auto();
        }
        Ok(())
    }

    fn force_restore_system_auto(&mut self) -> Result<(), String> {
        self.fan_actuation.restore_all()?;
        self.applied_targets.clear();
        self.manual_plan_active = false;
        self.system_auto_confirmed = true;
        Ok(())
    }

    fn recover_from_actuation_failure(&mut self) {
        let _ = self.fan_actuation.restore_all();
        self.applied_targets.clear();
        self.manual_plan_active = false;
        self.system_auto_confirmed = false;
    }

    pub(crate) fn update(
        &mut self,
        change: ThermalPolicyChange,
    ) -> Result<ThermalPolicySettings, String> {
        let restore_system_auto = matches!(
            &change,
            ThermalPolicyChange::SelectMode(ThermalPolicyMode::SystemAuto)
        );
        let mut updated = self.current.clone();
        match change {
            ThermalPolicyChange::SelectMode(mode) => updated.mode = mode,
            ThermalPolicyChange::UpsertRule(rule) => {
                if let Some(existing) = updated
                    .rules
                    .iter_mut()
                    .find(|existing| existing.id == rule.id)
                {
                    *existing = rule;
                } else {
                    updated.rules.push(rule);
                }
                updated.mode = ThermalPolicyMode::Custom;
            }
            ThermalPolicyChange::DeleteRule(rule_id) => {
                updated.rules.retain(|rule| rule.id != rule_id);
            }
        }

        settings::validate(&updated)?;
        self.store.save(&updated)?;
        self.current = updated.clone();
        self.evaluator = ThermalPolicyEvaluator::default();

        if restore_system_auto {
            self.force_restore_system_auto()?;
        }

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_telemetry::contract::{
        Availability, FanMode, FanReading, TemperatureReadings,
    };
    use crate::thermal_policy::contract::ThermalTarget;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Default)]
    struct MemoryStore {
        value: Rc<RefCell<Option<ThermalPolicySettings>>>,
        events: Rc<RefCell<Vec<&'static str>>>,
        fail_save: bool,
    }

    impl SettingsStore for MemoryStore {
        fn load(&self) -> Result<Option<ThermalPolicySettings>, String> {
            Ok(self.value.borrow().clone())
        }

        fn save(&mut self, settings: &ThermalPolicySettings) -> Result<(), String> {
            if self.fail_save {
                return Err("save failed".into());
            }
            self.events.borrow_mut().push("persist");
            *self.value.borrow_mut() = Some(settings.clone());
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct RecordingFanActuation {
        restore_count: Rc<RefCell<usize>>,
        direct_requests: Rc<RefCell<Vec<DirectFanActuationRequest>>>,
        events: Rc<RefCell<Vec<&'static str>>>,
        fail_direct: bool,
        fail_target_once: Rc<RefCell<bool>>,
        fail_heartbeat_once: Rc<RefCell<bool>>,
        fail_restore: bool,
    }

    impl FanActuation for RecordingFanActuation {
        fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String> {
            self.direct_requests
                .borrow_mut()
                .push(DirectFanActuationRequest::Target { fan_id, rpm });
            if self.fail_direct {
                return Err("direct actuation failed".into());
            }
            if *self.fail_target_once.borrow() {
                *self.fail_target_once.borrow_mut() = false;
                return Err("target failed".into());
            }
            Ok(())
        }

        fn system_auto(&mut self, fan_id: usize) -> Result<(), String> {
            self.direct_requests
                .borrow_mut()
                .push(DirectFanActuationRequest::SystemAuto { fan_id });
            if self.fail_direct {
                return Err("direct actuation failed".into());
            }
            Ok(())
        }

        fn restore_all(&mut self) -> Result<(), String> {
            self.events.borrow_mut().push("restore_all");
            *self.restore_count.borrow_mut() += 1;
            if self.fail_restore {
                return Err("restore failed".into());
            }
            Ok(())
        }

        fn heartbeat(&mut self) -> Result<(), String> {
            self.events.borrow_mut().push("heartbeat");
            if *self.fail_heartbeat_once.borrow() {
                *self.fail_heartbeat_once.borrow_mut() = false;
                return Err("heartbeat failed".into());
            }
            Ok(())
        }
    }

    fn snapshot(cpu_celsius: Option<f64>, captured_at_unix_ms: u64) -> HardwareTelemetrySnapshot {
        HardwareTelemetrySnapshot {
            temperatures: Availability::Available {
                value: TemperatureReadings {
                    cpu_celsius,
                    gpu_celsius: None,
                    sensors: vec![],
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
            captured_at_unix_ms,
        }
    }

    fn rule(id: &str) -> ThermalRule {
        ThermalRule {
            id: id.into(),
            name: "CPU".into(),
            target: ThermalTarget::Cpu,
            low_celsius: 40.0,
            high_celsius: 80.0,
            min_fan_percent: 20,
            max_fan_percent: 100,
            active: true,
        }
    }

    fn loaded(
        settings: ThermalPolicySettings,
    ) -> (
        ThermalPolicyState<MemoryStore, RecordingFanActuation>,
        MemoryStore,
        RecordingFanActuation,
    ) {
        let store = MemoryStore::default();
        *store.value.borrow_mut() = Some(settings);
        let shared_store = store.clone();
        let fan_actuation = RecordingFanActuation {
            events: store.events.clone(),
            ..Default::default()
        };
        let shared_fan_actuation = fan_actuation.clone();
        (
            ThermalPolicyState::load(store, fan_actuation),
            shared_store,
            shared_fan_actuation,
        )
    }

    #[test]
    fn startup_loads_persisted_settings_as_authoritative() {
        let persisted = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            rules: vec![],
        };
        let (state, _, _) = loaded(persisted.clone());
        assert_eq!(state.current(), persisted);
    }

    #[test]
    fn startup_uses_safe_defaults_for_invalid_persisted_settings() {
        let store = MemoryStore::default();
        let mut invalid = rule("cpu");
        invalid.low_celsius = 90.0;
        invalid.high_celsius = 80.0;
        *store.value.borrow_mut() = Some(ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![invalid],
        });

        let state = ThermalPolicyState::load(store, RecordingFanActuation::default());
        assert_eq!(state.current(), ThermalPolicySettings::default());
    }

    #[test]
    fn system_auto_snapshot_heartbeats_direct_target_without_redundant_restore() {
        let (mut state, _, fan_actuation) = loaded(ThermalPolicySettings::default());
        state
            .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
            .unwrap();
        state
            .direct_actuation(DirectFanActuationRequest::Target {
                fan_id: 0,
                rpm: 3000,
            })
            .unwrap();
        state
            .process_snapshot(&snapshot(Some(70.0), 2_000), 2_000)
            .unwrap();

        assert_eq!(*fan_actuation.restore_count.borrow(), 1);
        assert_eq!(
            fan_actuation
                .events
                .borrow()
                .iter()
                .filter(|event| **event == "heartbeat")
                .count(),
            2
        );
    }

    #[test]
    fn system_auto_heartbeat_failure_restores_and_returns_error() {
        let store = MemoryStore::default();
        let fan_actuation = RecordingFanActuation {
            events: store.events.clone(),
            fail_heartbeat_once: Rc::new(RefCell::new(true)),
            ..Default::default()
        };
        let shared = fan_actuation.clone();
        let mut state = ThermalPolicyState::load(store, fan_actuation);

        assert_eq!(
            state
                .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
                .unwrap_err(),
            "heartbeat failed"
        );
        assert_eq!(*shared.restore_count.borrow(), 2);
    }

    #[test]
    fn snapshot_processing_applies_changed_targets_and_heartbeats_without_persisting() {
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            rules: vec![],
        };
        let (mut state, store, fan_actuation) = loaded(settings.clone());

        let first = state
            .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
            .unwrap();
        let second = state
            .process_snapshot(&snapshot(Some(70.0), 2_000), 2_000)
            .unwrap();

        assert_eq!(
            first,
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 4657
                }]
            }
        );
        assert_eq!(second, first);
        assert_eq!(
            *fan_actuation.direct_requests.borrow(),
            vec![DirectFanActuationRequest::Target {
                fan_id: 0,
                rpm: 4657
            }]
        );
        assert_eq!(*store.events.borrow(), vec!["heartbeat", "heartbeat"]);
        assert_eq!(state.current(), settings);
    }

    #[test]
    fn fail_safe_snapshots_restore_system_auto_without_redundant_writes() {
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            rules: vec![],
        };
        let (mut state, _, fan_actuation) = loaded(settings);
        state
            .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
            .unwrap();

        let stale = state
            .process_snapshot(&snapshot(Some(70.0), 1_000), 6_001)
            .unwrap();
        let mut unavailable = snapshot(Some(70.0), 7_000);
        unavailable.temperatures = Availability::Unavailable {
            reason: "unavailable".into(),
        };
        let unavailable_plan = state.process_snapshot(&unavailable, 7_000).unwrap();
        let mut missing = snapshot(Some(70.0), 8_000);
        missing.fans = Availability::NotPresent;
        let missing_plan = state.process_snapshot(&missing, 8_000).unwrap();

        assert_eq!(stale, FanPlan::SystemAuto);
        assert_eq!(unavailable_plan, FanPlan::SystemAuto);
        assert_eq!(missing_plan, FanPlan::SystemAuto);
        assert_eq!(*fan_actuation.restore_count.borrow(), 1);
        assert_eq!(
            fan_actuation
                .events
                .borrow()
                .iter()
                .filter(|event| **event == "heartbeat")
                .count(),
            1
        );
    }

    #[test]
    fn target_and_heartbeat_failures_restore_and_recover_on_later_snapshots() {
        for fail_heartbeat in [false, true] {
            let settings = ThermalPolicySettings {
                mode: ThermalPolicyMode::Performance,
                rules: vec![],
            };
            let store = MemoryStore::default();
            *store.value.borrow_mut() = Some(settings);
            let fan_actuation = RecordingFanActuation {
                events: store.events.clone(),
                fail_target_once: Rc::new(RefCell::new(!fail_heartbeat)),
                fail_heartbeat_once: Rc::new(RefCell::new(fail_heartbeat)),
                ..Default::default()
            };
            let shared = fan_actuation.clone();
            let mut state = ThermalPolicyState::load(store, fan_actuation);

            let error = state
                .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
                .unwrap_err();
            assert_eq!(
                error,
                if fail_heartbeat {
                    "heartbeat failed"
                } else {
                    "target failed"
                }
            );
            assert_eq!(*shared.restore_count.borrow(), 1);

            let recovered = state
                .process_snapshot(&snapshot(Some(70.0), 2_000), 2_000)
                .unwrap();
            assert!(matches!(recovered, FanPlan::Targets { .. }));
            assert_eq!(
                shared
                    .direct_requests
                    .borrow()
                    .iter()
                    .filter(|request| matches!(request, DirectFanActuationRequest::Target { .. }))
                    .count(),
                2
            );
            assert_eq!(
                shared
                    .events
                    .borrow()
                    .iter()
                    .filter(|event| **event == "heartbeat")
                    .count(),
                if fail_heartbeat { 2 } else { 1 }
            );
        }
    }

    #[test]
    fn edited_custom_rule_updates_the_next_fan_plan() {
        let initial_rule = ThermalRule {
            min_fan_percent: 20,
            ..rule("cpu")
        };
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![initial_rule.clone()],
        };
        let (mut state, store, fan_actuation) = loaded(settings);

        assert_eq!(
            state
                .process_snapshot(&snapshot(Some(40.0), 1_000), 1_000)
                .unwrap(),
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 1800,
                }]
            }
        );

        state
            .update(ThermalPolicyChange::UpsertRule(ThermalRule {
                min_fan_percent: 10,
                ..initial_rule
            }))
            .unwrap();

        assert_eq!(state.current().rules[0].min_fan_percent, 10);
        assert_eq!(
            store.value.borrow().as_ref().unwrap().rules[0].min_fan_percent,
            10
        );
        assert_eq!(
            state
                .process_snapshot(&snapshot(Some(40.0), 2_000), 2_000)
                .unwrap(),
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 1400,
                }]
            }
        );
        assert_eq!(
            *fan_actuation.direct_requests.borrow(),
            vec![
                DirectFanActuationRequest::Target {
                    fan_id: 0,
                    rpm: 1800,
                },
                DirectFanActuationRequest::Target {
                    fan_id: 0,
                    rpm: 1400,
                },
            ]
        );
    }

    #[test]
    fn raising_custom_rule_updates_the_next_fan_plan() {
        let initial_rule = ThermalRule {
            min_fan_percent: 10,
            ..rule("cpu")
        };
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Custom,
            rules: vec![initial_rule.clone()],
        };
        let (mut state, _, _) = loaded(settings);

        assert_eq!(
            state
                .process_snapshot(&snapshot(Some(40.0), 1_000), 1_000)
                .unwrap(),
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 1400,
                }]
            }
        );
        state
            .update(ThermalPolicyChange::UpsertRule(ThermalRule {
                min_fan_percent: 20,
                ..initial_rule
            }))
            .unwrap();
        assert_eq!(
            state
                .process_snapshot(&snapshot(Some(40.0), 2_000), 2_000)
                .unwrap(),
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 1800,
                }]
            }
        );
    }

    #[test]
    fn failed_updates_preserve_evaluator_history() {
        for persistence_failure in [false, true] {
            let settings = ThermalPolicySettings {
                mode: ThermalPolicyMode::Performance,
                rules: vec![],
            };
            let store = MemoryStore {
                fail_save: persistence_failure,
                ..Default::default()
            };
            *store.value.borrow_mut() = Some(settings.clone());
            let fan_actuation = RecordingFanActuation {
                events: store.events.clone(),
                ..Default::default()
            };
            let mut state = ThermalPolicyState::load(store, fan_actuation);
            assert_eq!(
                state
                    .process_snapshot(&snapshot(Some(75.0), 1_000), 1_000)
                    .unwrap(),
                FanPlan::Targets {
                    targets: vec![super::super::contract::FanTarget {
                        fan_id: 0,
                        rpm: 5000,
                    }]
                }
            );

            let result = if persistence_failure {
                state.update(ThermalPolicyChange::SelectMode(ThermalPolicyMode::Quiet))
            } else {
                let mut invalid = rule("cpu");
                invalid.low_celsius = 90.0;
                invalid.high_celsius = 80.0;
                state.update(ThermalPolicyChange::UpsertRule(invalid))
            };
            assert!(result.is_err());
            assert_eq!(state.current(), settings);
            assert_eq!(
                state
                    .process_snapshot(&snapshot(Some(40.0), 2_000), 2_000)
                    .unwrap(),
                FanPlan::Targets {
                    targets: vec![super::super::contract::FanTarget {
                        fan_id: 0,
                        rpm: 4600,
                    }]
                }
            );
        }
    }

    #[test]
    fn direct_target_and_system_auto_succeed_in_system_auto_mode() {
        let settings = ThermalPolicySettings::default();
        let (mut state, store, fan_actuation) = loaded(settings.clone());

        state
            .direct_actuation(DirectFanActuationRequest::Target {
                fan_id: 1,
                rpm: 3200,
            })
            .unwrap();
        state
            .direct_actuation(DirectFanActuationRequest::SystemAuto { fan_id: 1 })
            .unwrap();

        assert_eq!(state.current(), settings);
        assert!(store.events.borrow().is_empty());
        assert_eq!(
            *fan_actuation.direct_requests.borrow(),
            vec![
                DirectFanActuationRequest::Target {
                    fan_id: 1,
                    rpm: 3200,
                },
                DirectFanActuationRequest::SystemAuto { fan_id: 1 },
            ]
        );
    }

    #[test]
    fn active_policy_modes_reject_direct_requests_without_actuation() {
        for mode in [
            ThermalPolicyMode::Quiet,
            ThermalPolicyMode::Performance,
            ThermalPolicyMode::Custom,
        ] {
            let (mut state, _, fan_actuation) = loaded(ThermalPolicySettings {
                mode,
                rules: vec![],
            });

            assert!(state
                .direct_actuation(DirectFanActuationRequest::Target {
                    fan_id: 0,
                    rpm: 3000,
                })
                .is_err());
            assert!(state
                .direct_actuation(DirectFanActuationRequest::SystemAuto { fan_id: 0 })
                .is_err());
            assert!(fan_actuation.direct_requests.borrow().is_empty());
        }
    }

    #[test]
    fn direct_actuation_failures_are_returned_without_mutating_settings() {
        for request in [
            DirectFanActuationRequest::Target {
                fan_id: 0,
                rpm: 3000,
            },
            DirectFanActuationRequest::SystemAuto { fan_id: 0 },
        ] {
            let settings = ThermalPolicySettings::default();
            let store = MemoryStore::default();
            let shared_store = store.clone();
            let fan_actuation = RecordingFanActuation {
                fail_direct: true,
                ..Default::default()
            };
            let mut state = ThermalPolicyState::load(store, fan_actuation);

            assert_eq!(
                state.direct_actuation(request).unwrap_err(),
                "direct actuation failed"
            );
            assert_eq!(state.current(), settings);
            assert!(shared_store.events.borrow().is_empty());
            assert!(shared_store.value.borrow().is_none());
        }
    }

    #[test]
    fn mode_selection_persists_and_commits_authoritative_settings() {
        let (mut state, store, _) = loaded(ThermalPolicySettings::default());
        let updated = state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::Performance,
            ))
            .unwrap();
        assert_eq!(updated.mode, ThermalPolicyMode::Performance);
        assert_eq!(state.current(), updated);
        assert_eq!(*store.value.borrow(), Some(updated));
    }

    #[test]
    fn rule_upsert_selects_custom_and_rule_deletion_persists() {
        let (mut state, store, _) = loaded(ThermalPolicySettings::default());
        let updated = state
            .update(ThermalPolicyChange::UpsertRule(rule("cpu")))
            .unwrap();
        assert_eq!(updated.mode, ThermalPolicyMode::Custom);
        assert_eq!(updated.rules, vec![rule("cpu")]);

        let deleted = state
            .update(ThermalPolicyChange::DeleteRule("cpu".into()))
            .unwrap();
        assert!(deleted.rules.is_empty());
        assert_eq!(*store.value.borrow(), Some(deleted));
    }

    #[test]
    fn validation_failure_leaves_state_unpersisted_and_performs_no_effect() {
        let (mut state, store, fan_actuation) = loaded(ThermalPolicySettings::default());
        let mut invalid = rule("cpu");
        invalid.low_celsius = 90.0;
        invalid.high_celsius = 80.0;

        assert!(state
            .update(ThermalPolicyChange::UpsertRule(invalid))
            .is_err());
        assert_eq!(state.current(), ThermalPolicySettings::default());
        assert_eq!(
            *store.value.borrow(),
            Some(ThermalPolicySettings::default())
        );
        assert_eq!(*fan_actuation.restore_count.borrow(), 0);
    }

    #[test]
    fn persistence_failure_leaves_state_unchanged_and_performs_no_effect() {
        let store = MemoryStore {
            fail_save: true,
            ..Default::default()
        };
        let fan_actuation = RecordingFanActuation::default();
        let shared_fan_actuation = fan_actuation.clone();
        let mut state = ThermalPolicyState::load(store, fan_actuation);

        assert!(state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::SystemAuto
            ))
            .is_err());
        assert_eq!(state.current(), ThermalPolicySettings::default());
        assert_eq!(*shared_fan_actuation.restore_count.borrow(), 0);
    }

    #[test]
    fn system_auto_mode_cycle_clears_evaluator_history() {
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            rules: vec![],
        };
        let (mut state, _, _) = loaded(settings);
        state
            .process_snapshot(&snapshot(Some(75.0), 1_000), 1_000)
            .unwrap();
        state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::SystemAuto,
            ))
            .unwrap();
        state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::Performance,
            ))
            .unwrap();

        let plan = state
            .process_snapshot(&snapshot(Some(40.0), 2_000), 2_000)
            .unwrap();
        assert_eq!(
            plan,
            FanPlan::Targets {
                targets: vec![super::super::contract::FanTarget {
                    fan_id: 0,
                    rpm: 2600,
                }]
            }
        );
    }

    #[test]
    fn explicit_system_auto_selection_clears_runtime_state() {
        let settings = ThermalPolicySettings {
            mode: ThermalPolicyMode::Performance,
            rules: vec![],
        };
        let (mut state, _, fan_actuation) = loaded(settings);
        state
            .process_snapshot(&snapshot(Some(70.0), 1_000), 1_000)
            .unwrap();

        state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::SystemAuto,
            ))
            .unwrap();
        state
            .process_snapshot(&snapshot(Some(70.0), 2_000), 2_000)
            .unwrap();

        assert_eq!(*fan_actuation.restore_count.borrow(), 1);
    }

    #[test]
    fn rule_deletion_while_system_auto_performs_no_actuation() {
        let (mut state, store, fan_actuation) = loaded(ThermalPolicySettings {
            mode: ThermalPolicyMode::SystemAuto,
            rules: vec![rule("cpu")],
        });

        let updated = state
            .update(ThermalPolicyChange::DeleteRule("cpu".into()))
            .unwrap();

        assert!(updated.rules.is_empty());
        assert_eq!(*store.value.borrow(), Some(updated));
        assert_eq!(*fan_actuation.restore_count.borrow(), 0);
    }

    #[test]
    fn persisted_system_auto_transition_commits_before_restoring_all_fans() {
        let initial = ThermalPolicySettings {
            mode: ThermalPolicyMode::Quiet,
            rules: vec![],
        };
        let (mut state, store, fan_actuation) = loaded(initial);
        let updated = state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::SystemAuto,
            ))
            .unwrap();

        assert_eq!(*store.value.borrow(), Some(updated.clone()));
        assert_eq!(state.current(), updated);
        assert_eq!(*fan_actuation.restore_count.borrow(), 1);
        assert_eq!(*store.events.borrow(), vec!["persist", "restore_all"]);
    }

    #[test]
    fn actuation_failure_keeps_the_persisted_authoritative_transition() {
        let store = MemoryStore::default();
        *store.value.borrow_mut() = Some(ThermalPolicySettings {
            mode: ThermalPolicyMode::Quiet,
            rules: vec![],
        });
        let shared_store = store.clone();
        let fan_actuation = RecordingFanActuation {
            events: store.events.clone(),
            fail_restore: true,
            ..Default::default()
        };
        let mut state = ThermalPolicyState::load(store, fan_actuation);

        assert!(state
            .update(ThermalPolicyChange::SelectMode(
                ThermalPolicyMode::SystemAuto,
            ))
            .is_err());
        assert_eq!(state.current().mode, ThermalPolicyMode::SystemAuto);
        assert_eq!(
            shared_store.value.borrow().as_ref().unwrap().mode,
            ThermalPolicyMode::SystemAuto
        );
    }
}
