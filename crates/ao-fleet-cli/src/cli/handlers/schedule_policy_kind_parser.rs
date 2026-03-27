use anyhow::{Result, anyhow};

use ao_fleet_core::SchedulePolicyKind;

pub fn parse_schedule_policy_kind(value: &str) -> Result<SchedulePolicyKind> {
    match value.trim() {
        "always_on" => Ok(SchedulePolicyKind::AlwaysOn),
        "business_hours" => Ok(SchedulePolicyKind::BusinessHours),
        "nightly" => Ok(SchedulePolicyKind::Nightly),
        "manual_only" => Ok(SchedulePolicyKind::ManualOnly),
        "burst_on_backlog" => Ok(SchedulePolicyKind::BurstOnBacklog),
        other => Err(anyhow!("unknown schedule policy kind: {other}")),
    }
}
