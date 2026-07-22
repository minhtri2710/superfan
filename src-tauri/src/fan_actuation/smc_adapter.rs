use super::protocol::{FanEnvelope, FanHardware};
use crate::smc::{
    getFanCount, getFanMaxSpeed, getFanMinSpeed, setFanAuto, setFanSpeed, with_smc_connection,
};

pub struct SmcFanHardware;

impl FanHardware for SmcFanHardware {
    fn envelopes(&self) -> Result<Vec<FanEnvelope>, String> {
        with_smc_connection(|connection| {
            let count = unsafe { getFanCount(connection) };
            if count <= 0 {
                return Err("no hardware fans are available".into());
            }

            let mut fans = Vec::with_capacity(count as usize);
            for id in 0..count {
                let min_rpm = unsafe { getFanMinSpeed(id, connection) };
                let max_rpm = unsafe { getFanMaxSpeed(id, connection) };
                if min_rpm <= 0.0 || max_rpm <= min_rpm {
                    return Err(format!("fan {id} has an invalid hardware range"));
                }
                fans.push(FanEnvelope {
                    id: id as usize,
                    min_rpm: min_rpm.round() as i32,
                    max_rpm: max_rpm.round() as i32,
                });
            }
            Ok(fans)
        })
    }

    fn set_target(&mut self, fan_id: usize, rpm: i32) -> Result<(), String> {
        with_smc_connection(|connection| {
            let result = unsafe { setFanSpeed(fan_id as i32, rpm, connection) };
            if result == 0 {
                Ok(())
            } else {
                Err(format!("SMC rejected target for fan {fan_id}: {result:#x}"))
            }
        })
    }

    fn system_auto(&mut self, fan_id: usize) -> Result<(), String> {
        with_smc_connection(|connection| {
            let result = unsafe { setFanAuto(fan_id as i32, connection) };
            if result == 0 {
                Ok(())
            } else {
                Err(format!("SMC could not restore fan {fan_id}: {result:#x}"))
            }
        })
    }
}
