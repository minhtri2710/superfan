use super::contract::{FanPlan, ThermalPolicySettings};
use super::evaluator::ThermalPolicyEvaluator;
use crate::fan_actuation::client;
use crate::hardware_telemetry::contract::HardwareTelemetrySnapshot;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ThermalPolicyRuntime {
    evaluator: ThermalPolicyEvaluator,
    applied_targets: BTreeMap<usize, i32>,
    manual_plan_active: bool,
}

impl ThermalPolicyRuntime {
    pub fn evaluate_and_apply(
        &mut self,
        settings: &ThermalPolicySettings,
        snapshot: &HardwareTelemetrySnapshot,
        now_unix_ms: u64,
    ) -> Result<FanPlan, String> {
        let plan = self.evaluator.evaluate(settings, snapshot, now_unix_ms);
        self.apply(&plan)?;
        Ok(plan)
    }

    pub fn restore_system_auto(&mut self) -> Result<(), String> {
        if self.manual_plan_active {
            client::restore_all()?;
        }
        self.applied_targets.clear();
        self.manual_plan_active = false;
        Ok(())
    }

    fn apply(&mut self, plan: &FanPlan) -> Result<(), String> {
        match plan {
            FanPlan::SystemAuto => self.restore_system_auto(),
            FanPlan::Targets { targets } => {
                let changed_targets = targets
                    .iter()
                    .filter(|target| self.applied_targets.get(&target.fan_id) != Some(&target.rpm))
                    .cloned()
                    .collect::<Vec<_>>();

                for target in &changed_targets {
                    if let Err(error) = client::set_target(target.fan_id, target.rpm) {
                        let _ = client::restore_all();
                        self.applied_targets.clear();
                        self.manual_plan_active = false;
                        return Err(error);
                    }
                }

                self.applied_targets = targets
                    .iter()
                    .map(|target| (target.fan_id, target.rpm))
                    .collect();
                self.manual_plan_active = true;
                if let Err(error) = client::heartbeat() {
                    let _ = client::restore_all();
                    self.applied_targets.clear();
                    self.manual_plan_active = false;
                    return Err(error);
                }
                Ok(())
            }
        }
    }
}
