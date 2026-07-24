use super::contract::{ThermalPolicyMode, ThermalPolicySettings, ThermalRule};
use super::settings;

pub(crate) trait SettingsStore {
    fn load(&self) -> Result<Option<ThermalPolicySettings>, String>;
    fn save(&mut self, settings: &ThermalPolicySettings) -> Result<(), String>;
}

pub(crate) trait FanActuation {
    fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String>;
    fn system_auto(&mut self, fan_id: usize) -> Result<(), String>;
    fn restore_all(&mut self) -> Result<(), String>;
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

        if restore_system_auto {
            self.fan_actuation.restore_all()?;
        }

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
