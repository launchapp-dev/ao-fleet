use ao_fleet_core::{
    DaemonDesiredState, DaemonOverride, Schedule, SchedulePolicyKind, WeekdayWindow,
};
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamReconcileEvaluation {
    pub team_id: String,
    pub desired_state: DaemonDesiredState,
    pub reason: String,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
    pub override_applied: Option<DaemonOverride>,
}

impl TeamReconcileEvaluation {
    pub fn evaluate(
        team_id: impl Into<String>,
        schedules: &[Schedule],
        override_applied: Option<&DaemonOverride>,
        evaluated_at: DateTime<Utc>,
        backlog_count: usize,
    ) -> Self {
        let team_id = team_id.into();
        let schedule_ids = schedules.iter().map(|schedule| schedule.id.clone()).collect();

        if let Some(override_applied) = override_applied.filter(|override_applied| {
            override_applied.team_id == team_id && override_applied.is_active(evaluated_at)
        }) {
            let desired_state = override_applied.forced_state.unwrap_or(DaemonDesiredState::Paused);
            return Self {
                team_id,
                desired_state,
                reason: override_reason(override_applied),
                backlog_count,
                schedule_ids,
                override_applied: Some(override_applied.clone()),
            };
        }

        let mut desired_state = DaemonDesiredState::Stopped;
        let mut reason = if schedules.is_empty() {
            "no schedules configured for this team".to_string()
        } else {
            "all enabled schedules keep the daemon stopped".to_string()
        };
        let mut best_rank = state_rank(desired_state);
        let mut enabled_schedule_count = 0_usize;

        for schedule in schedules {
            if !schedule.enabled {
                continue;
            }

            enabled_schedule_count += 1;
            let candidate_state =
                ScheduleEvaluator::evaluate(schedule, evaluated_at, backlog_count);
            let candidate_reason =
                schedule_reason(schedule, evaluated_at, backlog_count, candidate_state);
            let candidate_rank = state_rank(candidate_state);

            if candidate_rank > best_rank {
                desired_state = candidate_state;
                reason = candidate_reason;
                best_rank = candidate_rank;
            }
        }

        if schedules.is_empty() {
            desired_state = DaemonDesiredState::Stopped;
            reason = "no schedules configured for this team".to_string();
        } else if enabled_schedule_count == 0 {
            desired_state = DaemonDesiredState::Stopped;
            reason = "all schedules are disabled for this team".to_string();
        }

        Self { team_id, desired_state, reason, backlog_count, schedule_ids, override_applied: None }
    }
}

fn override_reason(override_applied: &DaemonOverride) -> String {
    let note = override_applied
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!(" ({value})"))
        .unwrap_or_default();

    match override_applied.mode {
        ao_fleet_core::DaemonOverrideMode::ForceDesiredState => format!(
            "founder override from {} forcing daemon {}{}",
            override_applied.source,
            desired_state_label(
                override_applied.forced_state.unwrap_or(DaemonDesiredState::Paused)
            ),
            note
        ),
        ao_fleet_core::DaemonOverrideMode::FreezeUntil => format!(
            "founder override from {} freezing daemon paused until {}{}",
            override_applied.source,
            override_applied.pause_until.unwrap_or_else(Utc::now).to_rfc3339(),
            note
        ),
    }
}

fn schedule_reason(
    schedule: &Schedule,
    evaluated_at: DateTime<Utc>,
    backlog_count: usize,
    desired_state: DaemonDesiredState,
) -> String {
    let local_time = to_local_time(schedule, evaluated_at);
    match schedule.policy_kind {
        SchedulePolicyKind::AlwaysOn => {
            format!("always_on schedule keeps the daemon running at {}", local_time.to_rfc3339())
        }
        SchedulePolicyKind::BusinessHours => {
            let window = matching_weekday_window(schedule, local_time)
                .or_else(|| schedule.windows.first())
                .map(window_summary)
                .unwrap_or_else(|| "configured business hours".to_string());

            if desired_state == DaemonDesiredState::Running {
                format!("business_hours schedule is within {} in {}", window, schedule.timezone)
            } else {
                format!("business_hours schedule is outside {} in {}", window, schedule.timezone)
            }
        }
        SchedulePolicyKind::Nightly => {
            let window = schedule
                .windows
                .first()
                .map(window_summary)
                .unwrap_or_else(|| "configured nightly window".to_string());

            if desired_state == DaemonDesiredState::Running {
                format!("nightly schedule is within {} in {}", window, schedule.timezone)
            } else {
                format!("nightly schedule is outside {} in {}", window, schedule.timezone)
            }
        }
        SchedulePolicyKind::ManualOnly => {
            "manual_only schedule keeps the daemon stopped until a founder override is applied"
                .to_string()
        }
        SchedulePolicyKind::BurstOnBacklog => {
            if backlog_count > 0 {
                format!(
                    "burst_on_backlog schedule saw {} backlog item(s) and is running",
                    backlog_count
                )
            } else {
                "burst_on_backlog schedule has no backlog and stays paused".to_string()
            }
        }
    }
}

fn matching_weekday_window(
    schedule: &Schedule,
    local_time: DateTime<Tz>,
) -> Option<&WeekdayWindow> {
    let weekday = local_time.weekday().num_days_from_monday() as u8;
    let hour = local_time.hour() as u8;

    schedule.windows.iter().find(|window| {
        window.weekdays.contains(&weekday) && hour >= window.start_hour && hour < window.end_hour
    })
}

fn state_rank(state: DaemonDesiredState) -> u8 {
    match state {
        DaemonDesiredState::Running => 2,
        DaemonDesiredState::Paused => 1,
        DaemonDesiredState::Stopped => 0,
    }
}

fn desired_state_label(state: DaemonDesiredState) -> &'static str {
    match state {
        DaemonDesiredState::Running => "running",
        DaemonDesiredState::Paused => "paused",
        DaemonDesiredState::Stopped => "stopped",
    }
}

fn to_local_time(schedule: &Schedule, at: DateTime<Utc>) -> DateTime<Tz> {
    let timezone = schedule.timezone.parse::<Tz>().unwrap_or(chrono_tz::UTC);
    at.with_timezone(&timezone)
}

fn window_summary(window: &WeekdayWindow) -> String {
    if window.weekdays.is_empty() {
        return format!("{:02}:00-{:02}:00", window.start_hour, window.end_hour);
    }

    let weekdays =
        window.weekdays.iter().map(|weekday| weekday_label(*weekday)).collect::<Vec<_>>().join(",");
    format!("{weekdays} {:02}:00-{:02}:00", window.start_hour, window.end_hour)
}

fn weekday_label(weekday: u8) -> &'static str {
    match weekday {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        6 => "Sun",
        _ => "Unknown",
    }
}
