use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use ao_fleet_ao::{AoDaemonClient, DaemonCommandResult, DaemonStartOptions, DaemonState};
use ao_fleet_core::DaemonDesiredState;

use super::daemon_reconcile_result::DaemonReconcileResult;

const CONTROL_RETRY_ATTEMPTS: usize = 3;
const STATUS_RETRY_ATTEMPTS: usize = 3;
const RETRY_BASE_DELAY_MS: u64 = 50;

pub(crate) trait DaemonController {
    fn daemon_status(&self, project_root: &str) -> Result<DaemonState>;
    fn project_status(&self, project_root: &str) -> Result<DaemonState>;
    fn start_daemon(&self, project_root: &str) -> Result<DaemonCommandResult>;
    fn resume_daemon(&self, project_root: &str) -> Result<DaemonCommandResult>;
    fn pause_daemon(&self, project_root: &str) -> Result<DaemonCommandResult>;
    fn stop_daemon(&self, project_root: &str) -> Result<DaemonCommandResult>;
}

impl DaemonController for AoDaemonClient {
    fn daemon_status(&self, project_root: &str) -> Result<DaemonState> {
        AoDaemonClient::daemon_status(self, project_root).map_err(Into::into)
    }

    fn project_status(&self, project_root: &str) -> Result<DaemonState> {
        AoDaemonClient::project_status(self, project_root)
            .map(|report| report.daemon_state)
            .map_err(Into::into)
    }

    fn start_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        AoDaemonClient::start(self, project_root, &DaemonStartOptions::default())
            .map_err(Into::into)
    }

    fn resume_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        AoDaemonClient::resume(self, project_root).map_err(Into::into)
    }

    fn pause_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        AoDaemonClient::pause(self, project_root).map_err(Into::into)
    }

    fn stop_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        AoDaemonClient::stop(self, project_root, None).map_err(Into::into)
    }
}

pub(crate) fn reconcile_project<C: DaemonController + ?Sized>(
    controller: &C,
    team_id: String,
    project_id: String,
    project_root: String,
    target: serde_json::Value,
    desired_state: DaemonDesiredState,
    backlog_count: usize,
    schedule_ids: Vec<String>,
    apply: bool,
) -> Result<DaemonReconcileResult> {
    let observed_state = resolve_observed_state(controller, &project_root);
    let action = planned_action(desired_state, observed_state.clone()).map(str::to_string);
    let command_result =
        if apply { execute_action(controller, &project_root, action.as_deref())? } else { None };

    let refreshed_state = if apply {
        resolve_observed_state(controller, &project_root)
    } else {
        observed_state.clone()
    };

    let observed_state = refreshed_state.or_else(|| action.as_deref().and_then(action_to_state));

    Ok(DaemonReconcileResult {
        team_id,
        project_id,
        project_root,
        target,
        desired_state,
        observed_state,
        backlog_count,
        schedule_ids,
        action,
        command_result,
    })
}

fn resolve_observed_state<C: DaemonController + ?Sized>(
    controller: &C,
    project_root: &str,
) -> Option<DaemonState> {
    let daemon_state =
        retry_with_backoff(STATUS_RETRY_ATTEMPTS, sleep, || controller.daemon_status(project_root))
            .ok();

    match daemon_state {
        Some(state) if !is_stale_state(&state) => Some(state),
        _ => retry_with_backoff(STATUS_RETRY_ATTEMPTS, sleep, || {
            controller.project_status(project_root)
        })
        .ok()
        .filter(|state| !is_stale_state(state)),
    }
}

fn execute_action<C: DaemonController + ?Sized>(
    controller: &C,
    project_root: &str,
    action: Option<&str>,
) -> Result<Option<DaemonCommandResult>> {
    let Some(action) = action else {
        return Ok(None);
    };

    let result = retry_with_backoff(CONTROL_RETRY_ATTEMPTS, sleep, || match action {
        "start" => controller.start_daemon(project_root),
        "resume" => controller.resume_daemon(project_root),
        "pause" => controller.pause_daemon(project_root),
        "stop" => controller.stop_daemon(project_root),
        other => bail!("unsupported daemon action: {other}"),
    })?;

    Ok(Some(result))
}

fn planned_action(
    desired_state: DaemonDesiredState,
    observed_state: Option<DaemonState>,
) -> Option<&'static str> {
    match (desired_state, observed_state) {
        (DaemonDesiredState::Running, Some(DaemonState::Running)) => None,
        (DaemonDesiredState::Running, Some(DaemonState::Paused)) => Some("resume"),
        (DaemonDesiredState::Running, _) => Some("start"),
        (DaemonDesiredState::Paused, Some(DaemonState::Running)) => Some("pause"),
        (DaemonDesiredState::Paused, _) => None,
        (DaemonDesiredState::Stopped, Some(DaemonState::Running | DaemonState::Paused)) => {
            Some("stop")
        }
        (DaemonDesiredState::Stopped, _) => Some("stop"),
    }
}

fn action_to_state(action: &str) -> Option<DaemonState> {
    match action {
        "start" | "resume" => Some(DaemonState::Running),
        "pause" => Some(DaemonState::Paused),
        "stop" => Some(DaemonState::Stopped),
        _ => None,
    }
}

fn is_stale_state(state: &DaemonState) -> bool {
    matches!(state, DaemonState::Crashed | DaemonState::Unknown(_))
}

fn retry_with_backoff<T, F, S>(attempts: usize, mut sleeper: S, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
    S: FnMut(Duration),
{
    let attempts = attempts.max(1);
    let mut last_error = None;

    for attempt in 0..attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < attempts {
                    sleeper(retry_delay(attempt));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("operation failed without an error")))
}

fn retry_delay(attempt: usize) -> Duration {
    Duration::from_millis(RETRY_BASE_DELAY_MS.saturating_mul(1_u64 << attempt.min(20)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    fn command_result(command: &str, state: Option<DaemonState>) -> DaemonCommandResult {
        DaemonCommandResult {
            command: command.to_string(),
            message: Some(format!("{command} complete")),
            daemon_pid: Some(4242),
            state: state.clone(),
            raw: serde_json::json!({
                "command": command,
                "state": state.as_ref().map(|value| String::from(value.clone())),
            }),
        }
    }

    #[derive(Default)]
    struct FakeDaemonController {
        daemon_statuses: RefCell<VecDeque<Result<DaemonState, &'static str>>>,
        project_statuses: RefCell<VecDeque<Result<DaemonState, &'static str>>>,
        start_results: RefCell<VecDeque<Result<DaemonCommandResult, &'static str>>>,
        resume_results: RefCell<VecDeque<Result<DaemonCommandResult, &'static str>>>,
        pause_results: RefCell<VecDeque<Result<DaemonCommandResult, &'static str>>>,
        stop_results: RefCell<VecDeque<Result<DaemonCommandResult, &'static str>>>,
        calls: RefCell<Vec<String>>,
    }

    impl FakeDaemonController {
        fn push_daemon_status(&self, state: Result<DaemonState, &'static str>) {
            self.daemon_statuses.borrow_mut().push_back(state);
        }

        fn push_project_status(&self, state: Result<DaemonState, &'static str>) {
            self.project_statuses.borrow_mut().push_back(state);
        }

        fn push_action_result(
            &self,
            action: &'static str,
            result: Result<DaemonCommandResult, &'static str>,
        ) {
            match action {
                "start" => self.start_results.borrow_mut().push_back(result),
                "resume" => self.resume_results.borrow_mut().push_back(result),
                "pause" => self.pause_results.borrow_mut().push_back(result),
                "stop" => self.stop_results.borrow_mut().push_back(result),
                other => panic!("unsupported action in test fixture: {other}"),
            }
        }

        fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }

        fn next_status(
            queue: &RefCell<VecDeque<Result<DaemonState, &'static str>>>,
        ) -> Result<DaemonState> {
            queue
                .borrow_mut()
                .pop_front()
                .ok_or_else(|| anyhow!("missing status fixture"))?
                .map_err(anyhow::Error::msg)
        }

        fn next_action(
            queue: &RefCell<VecDeque<Result<DaemonCommandResult, &'static str>>>,
        ) -> Result<DaemonCommandResult> {
            queue
                .borrow_mut()
                .pop_front()
                .ok_or_else(|| anyhow!("missing action fixture"))?
                .map_err(anyhow::Error::msg)
        }
    }

    impl DaemonController for FakeDaemonController {
        fn daemon_status(&self, _project_root: &str) -> Result<DaemonState> {
            self.calls.borrow_mut().push("daemon_status".to_string());
            Self::next_status(&self.daemon_statuses)
        }

        fn project_status(&self, _project_root: &str) -> Result<DaemonState> {
            self.calls.borrow_mut().push("project_status".to_string());
            Self::next_status(&self.project_statuses)
        }

        fn start_daemon(&self, _project_root: &str) -> Result<DaemonCommandResult> {
            self.calls.borrow_mut().push("start".to_string());
            Self::next_action(&self.start_results)
        }

        fn resume_daemon(&self, _project_root: &str) -> Result<DaemonCommandResult> {
            self.calls.borrow_mut().push("resume".to_string());
            Self::next_action(&self.resume_results)
        }

        fn pause_daemon(&self, _project_root: &str) -> Result<DaemonCommandResult> {
            self.calls.borrow_mut().push("pause".to_string());
            Self::next_action(&self.pause_results)
        }

        fn stop_daemon(&self, _project_root: &str) -> Result<DaemonCommandResult> {
            self.calls.borrow_mut().push("stop".to_string());
            Self::next_action(&self.stop_results)
        }
    }

    #[test]
    fn retry_with_backoff_records_sleep_and_retries() {
        let mut attempts = 0;
        let mut sleeps = Vec::new();
        let result = retry_with_backoff(
            3,
            |delay| sleeps.push(delay),
            || {
                attempts += 1;
                if attempts < 3 { bail!("transient error") } else { Ok("ok") }
            },
        )
        .expect("retry eventually succeeds");

        assert_eq!(result, "ok");
        assert_eq!(attempts, 3);
        assert_eq!(sleeps.len(), 2);
        assert_eq!(sleeps[0], retry_delay(0));
        assert_eq!(sleeps[1], retry_delay(1));
    }

    #[test]
    fn reconcile_project_prefers_project_status_when_daemon_state_is_stale() {
        let controller = FakeDaemonController::default();
        controller.push_daemon_status(Ok(DaemonState::Unknown("stale".to_string())));
        controller.push_project_status(Ok(DaemonState::Running));

        let result = reconcile_project(
            &controller,
            "team-1".to_string(),
            "project-1".to_string(),
            "/tmp/project".to_string(),
            serde_json::json!({"transport": "local_cli"}),
            DaemonDesiredState::Running,
            7,
            vec!["schedule-1".to_string()],
            false,
        )
        .expect("reconcile succeeds");

        assert_eq!(result.observed_state, Some(DaemonState::Running));
        assert_eq!(result.action, None);
        assert!(result.command_result.is_none());
        assert_eq!(
            controller.calls(),
            vec!["daemon_status".to_string(), "project_status".to_string()]
        );
    }

    #[test]
    fn reconcile_project_retries_action_and_uses_action_state_when_refresh_fails() {
        let controller = FakeDaemonController::default();
        controller.push_daemon_status(Err("daemon unavailable"));
        controller.push_daemon_status(Err("daemon unavailable"));
        controller.push_daemon_status(Err("daemon unavailable"));
        controller.push_project_status(Err("project unavailable"));
        controller.push_project_status(Err("project unavailable"));
        controller.push_project_status(Err("project unavailable"));
        controller.push_action_result("start", Err("transient start failure"));
        controller
            .push_action_result("start", Ok(command_result("start", Some(DaemonState::Running))));
        controller.push_daemon_status(Err("post-action daemon unavailable"));
        controller.push_daemon_status(Err("post-action daemon unavailable"));
        controller.push_daemon_status(Err("post-action daemon unavailable"));
        controller.push_project_status(Err("post-action project unavailable"));
        controller.push_project_status(Err("post-action project unavailable"));
        controller.push_project_status(Err("post-action project unavailable"));

        let result = reconcile_project(
            &controller,
            "team-1".to_string(),
            "project-1".to_string(),
            "/tmp/project".to_string(),
            serde_json::json!({"transport": "local_cli"}),
            DaemonDesiredState::Running,
            7,
            vec!["schedule-1".to_string()],
            true,
        )
        .expect("reconcile succeeds");

        assert_eq!(result.action.as_deref(), Some("start"));
        assert_eq!(result.observed_state, Some(DaemonState::Running));
        assert_eq!(
            result.command_result.as_ref().map(|value| value.command.as_str()),
            Some("start")
        );
        assert_eq!(
            controller.calls(),
            vec![
                "daemon_status".to_string(),
                "daemon_status".to_string(),
                "daemon_status".to_string(),
                "project_status".to_string(),
                "project_status".to_string(),
                "project_status".to_string(),
                "start".to_string(),
                "start".to_string(),
                "daemon_status".to_string(),
                "daemon_status".to_string(),
                "daemon_status".to_string(),
                "project_status".to_string(),
                "project_status".to_string(),
                "project_status".to_string(),
            ]
        );
    }
}
