use std::ffi::OsString;
use std::path::Path;

use crate::models::daemon_start_options::DaemonStartOptions;

pub(crate) fn build_daemon_status_args(project_root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("daemon"),
        OsString::from("status"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ]
}

pub(crate) fn build_daemon_start_args(
    project_root: &Path,
    options: &DaemonStartOptions,
) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("daemon"),
        OsString::from("start"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ];

    if options.autonomous {
        args.push(OsString::from("--autonomous"));
    }
    if options.skip_runner {
        args.push(OsString::from("--skip-runner"));
    }
    if let Some(pool_size) = options.pool_size {
        args.push(OsString::from("--pool-size"));
        args.push(pool_size.to_string().into());
    }
    if let Some(interval_secs) = options.interval_secs {
        args.push(OsString::from("--interval-secs"));
        args.push(interval_secs.to_string().into());
    }
    if let Some(value) = options.auto_run_ready {
        args.push(OsString::from("--auto-run-ready"));
        args.push(value.to_string().into());
    }

    args
}

pub(crate) fn build_daemon_stop_args(
    project_root: &Path,
    shutdown_timeout_secs: Option<u64>,
) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("daemon"),
        OsString::from("stop"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ];

    if let Some(timeout) = shutdown_timeout_secs {
        args.push(OsString::from("--shutdown-timeout-secs"));
        args.push(timeout.to_string().into());
    }

    args
}

pub(crate) fn build_daemon_pause_args(project_root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("daemon"),
        OsString::from("pause"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ]
}

pub(crate) fn build_daemon_resume_args(project_root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("daemon"),
        OsString::from("resume"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ]
}

pub(crate) fn build_project_status_args(project_root: &Path) -> Vec<OsString> {
    vec![
        OsString::from("status"),
        OsString::from("--json"),
        OsString::from("--project-root"),
        project_root.as_os_str().to_os_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_start_includes_expected_flags() {
        let options = DaemonStartOptions {
            autonomous: true,
            skip_runner: true,
            pool_size: Some(3),
            interval_secs: Some(15),
            auto_run_ready: Some(true),
        };

        let args = build_daemon_start_args(Path::new("/tmp/project"), &options);
        let rendered =
            args.into_iter().map(|arg| arg.to_string_lossy().to_string()).collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "daemon",
                "start",
                "--json",
                "--project-root",
                "/tmp/project",
                "--autonomous",
                "--skip-runner",
                "--pool-size",
                "3",
                "--interval-secs",
                "15",
                "--auto-run-ready",
                "true",
            ]
        );
    }

    #[test]
    fn daemon_stop_includes_optional_timeout() {
        let args = build_daemon_stop_args(Path::new("/tmp/project"), Some(60));
        let rendered =
            args.into_iter().map(|arg| arg.to_string_lossy().to_string()).collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "daemon",
                "stop",
                "--json",
                "--project-root",
                "/tmp/project",
                "--shutdown-timeout-secs",
                "60",
            ]
        );
    }
}
