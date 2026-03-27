use ao_fleet_core::{DaemonDesiredState, Schedule, SchedulePolicyKind, WeekdayWindow};
use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;

#[derive(Debug, Default, Clone, Copy)]
pub struct ScheduleEvaluator;

impl ScheduleEvaluator {
    pub fn evaluate(
        schedule: &Schedule,
        at: DateTime<Utc>,
        backlog_count: usize,
    ) -> DaemonDesiredState {
        if !schedule.enabled {
            return DaemonDesiredState::Stopped;
        }

        match schedule.policy_kind {
            SchedulePolicyKind::AlwaysOn => DaemonDesiredState::Running,
            SchedulePolicyKind::ManualOnly => DaemonDesiredState::Stopped,
            SchedulePolicyKind::BurstOnBacklog => {
                if backlog_count > 0 {
                    DaemonDesiredState::Running
                } else {
                    DaemonDesiredState::Paused
                }
            }
            SchedulePolicyKind::BusinessHours => {
                if is_within_business_hours(schedule, at) {
                    DaemonDesiredState::Running
                } else {
                    DaemonDesiredState::Paused
                }
            }
            SchedulePolicyKind::Nightly => {
                if is_within_nightly_window(schedule, at) {
                    DaemonDesiredState::Running
                } else {
                    DaemonDesiredState::Paused
                }
            }
        }
    }
}

fn is_within_business_hours(schedule: &Schedule, at: DateTime<Utc>) -> bool {
    let local = to_local_time(schedule, at);
    let weekday = local.weekday().num_days_from_monday() as u8;
    let hour = local.hour() as u8;

    schedule.windows.iter().any(|window| matches_weekday_window(window, weekday, hour))
}

fn is_within_nightly_window(schedule: &Schedule, at: DateTime<Utc>) -> bool {
    let local = to_local_time(schedule, at);
    let hour = local.hour() as u8;

    schedule.windows.iter().any(|window| matches_nightly_window(window, hour))
}

fn matches_weekday_window(window: &WeekdayWindow, weekday: u8, hour: u8) -> bool {
    window.weekdays.contains(&weekday) && hour >= window.start_hour && hour < window.end_hour
}

fn matches_nightly_window(window: &WeekdayWindow, hour: u8) -> bool {
    if window.start_hour <= window.end_hour {
        return hour >= window.start_hour && hour < window.end_hour;
    }

    hour >= window.start_hour || hour < window.end_hour
}

fn to_local_time(schedule: &Schedule, at: DateTime<Utc>) -> DateTime<Tz> {
    let timezone = schedule.timezone.parse::<Tz>().unwrap_or(chrono_tz::UTC);
    at.with_timezone(&timezone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn business_schedule() -> Schedule {
        Schedule {
            id: "schedule-1".to_string(),
            team_id: "team-1".to_string(),
            timezone: "UTC".to_string(),
            policy_kind: SchedulePolicyKind::BusinessHours,
            windows: vec![WeekdayWindow {
                weekdays: vec![0, 1, 2, 3, 4],
                start_hour: 9,
                end_hour: 17,
            }],
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn nightly_schedule() -> Schedule {
        Schedule {
            id: "schedule-2".to_string(),
            team_id: "team-1".to_string(),
            timezone: "UTC".to_string(),
            policy_kind: SchedulePolicyKind::Nightly,
            windows: vec![WeekdayWindow { weekdays: vec![], start_hour: 22, end_hour: 6 }],
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn weekday_window_matches_only_within_range() {
        let window = WeekdayWindow { weekdays: vec![0, 2, 4], start_hour: 9, end_hour: 17 };

        assert!(matches_weekday_window(&window, 0, 9));
        assert!(matches_weekday_window(&window, 2, 16));
        assert!(!matches_weekday_window(&window, 1, 10));
        assert!(!matches_weekday_window(&window, 4, 17));
    }

    #[test]
    fn always_on_is_running() {
        let schedule =
            Schedule { policy_kind: SchedulePolicyKind::AlwaysOn, ..business_schedule() };

        let state = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 1, 0, 0).unwrap(),
            0,
        );
        assert_eq!(state, DaemonDesiredState::Running);
    }

    #[test]
    fn manual_only_is_stopped() {
        let schedule =
            Schedule { policy_kind: SchedulePolicyKind::ManualOnly, ..business_schedule() };

        let state = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
            42,
        );
        assert_eq!(state, DaemonDesiredState::Stopped);
    }

    #[test]
    fn business_hours_uses_window_and_timezone() {
        let schedule = business_schedule();

        let running = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
            0,
        );
        let paused = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 20, 0, 0).unwrap(),
            0,
        );

        assert_eq!(running, DaemonDesiredState::Running);
        assert_eq!(paused, DaemonDesiredState::Paused);
    }

    #[test]
    fn nightly_supports_wraparound_window() {
        let schedule = nightly_schedule();

        let running_late = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 23, 0, 0).unwrap(),
            0,
        );
        let running_early = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 4, 2, 0, 0).unwrap(),
            0,
        );
        let paused_day = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 4, 12, 0, 0).unwrap(),
            0,
        );

        assert_eq!(running_late, DaemonDesiredState::Running);
        assert_eq!(running_early, DaemonDesiredState::Running);
        assert_eq!(paused_day, DaemonDesiredState::Paused);
    }

    #[test]
    fn burst_on_backlog_runs_only_when_backlog_exists() {
        let schedule =
            Schedule { policy_kind: SchedulePolicyKind::BurstOnBacklog, ..business_schedule() };

        let idle = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
            0,
        );
        let burst = ScheduleEvaluator::evaluate(
            &schedule,
            Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
            1,
        );

        assert_eq!(idle, DaemonDesiredState::Paused);
        assert_eq!(burst, DaemonDesiredState::Running);
    }
}
