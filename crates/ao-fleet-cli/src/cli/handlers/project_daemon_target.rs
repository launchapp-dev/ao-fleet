use std::collections::BTreeMap;

use anyhow::Result;
use ao_fleet_ao::{AoDaemonClient, AoRemoteDaemonClient, DaemonCommandResult, DaemonState};
use ao_fleet_core::{Host, Project, ProjectHostPlacement};

use crate::cli::handlers::daemon_reconcile_support::DaemonController;

#[derive(Debug, Clone)]
pub(crate) struct ProjectDaemonTarget {
    controller: ProjectDaemonController,
    transport: String,
    host_id: Option<String>,
    host_slug: Option<String>,
    host_address: Option<String>,
    resolution: String,
}

impl ProjectDaemonTarget {
    pub(crate) fn local() -> Self {
        Self {
            controller: ProjectDaemonController::Local(AoDaemonClient::new()),
            transport: "local_cli".to_string(),
            host_id: None,
            host_slug: None,
            host_address: None,
            resolution: "no_host_placement".to_string(),
        }
    }

    pub(crate) fn source_name(&self) -> &'static str {
        match self.controller {
            ProjectDaemonController::Local(_) => "ao-cli",
            ProjectDaemonController::Remote(_) => "ao-web-api",
        }
    }

    pub(crate) fn controller(&self) -> &ProjectDaemonController {
        &self.controller
    }

    pub(crate) fn details(&self) -> serde_json::Value {
        serde_json::json!({
            "transport": self.transport,
            "host_id": self.host_id,
            "host_slug": self.host_slug,
            "host_address": self.host_address,
            "resolution": self.resolution,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ProjectDaemonController {
    Local(AoDaemonClient),
    Remote(AoRemoteDaemonClient),
}

impl DaemonController for ProjectDaemonController {
    fn daemon_status(&self, project_root: &str) -> Result<DaemonState> {
        match self {
            ProjectDaemonController::Local(client) => {
                client.daemon_status(project_root).map_err(Into::into)
            }
            ProjectDaemonController::Remote(client) => client.daemon_status(),
        }
    }

    fn project_status(&self, project_root: &str) -> Result<DaemonState> {
        match self {
            ProjectDaemonController::Local(client) => client
                .project_status(project_root)
                .map(|report| report.daemon_state)
                .map_err(Into::into),
            ProjectDaemonController::Remote(client) => client.daemon_status(),
        }
    }

    fn start_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        match self {
            ProjectDaemonController::Local(client) => client
                .start(project_root, &ao_fleet_ao::DaemonStartOptions::default())
                .map_err(Into::into),
            ProjectDaemonController::Remote(client) => client.start(),
        }
    }

    fn resume_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        match self {
            ProjectDaemonController::Local(client) => {
                client.resume(project_root).map_err(Into::into)
            }
            ProjectDaemonController::Remote(client) => client.resume(),
        }
    }

    fn pause_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        match self {
            ProjectDaemonController::Local(client) => {
                client.pause(project_root).map_err(Into::into)
            }
            ProjectDaemonController::Remote(client) => client.pause(),
        }
    }

    fn stop_daemon(&self, project_root: &str) -> Result<DaemonCommandResult> {
        match self {
            ProjectDaemonController::Local(client) => {
                client.stop(project_root, None).map_err(Into::into)
            }
            ProjectDaemonController::Remote(client) => client.stop(),
        }
    }
}

pub(crate) fn build_host_map(hosts: Vec<Host>) -> BTreeMap<String, Host> {
    hosts.into_iter().map(|host| (host.id.clone(), host)).collect()
}

pub(crate) fn build_project_host_placement_map(
    placements: Vec<ProjectHostPlacement>,
) -> BTreeMap<String, ProjectHostPlacement> {
    placements.into_iter().map(|placement| (placement.project_id.clone(), placement)).collect()
}

pub(crate) fn resolve_project_daemon_target(
    project: &Project,
    placement_map: &BTreeMap<String, ProjectHostPlacement>,
    host_map: &BTreeMap<String, Host>,
) -> ProjectDaemonTarget {
    let Some(placement) = placement_map.get(&project.id) else {
        return ProjectDaemonTarget::local();
    };

    let Some(host) = host_map.get(&placement.host_id) else {
        let mut target = ProjectDaemonTarget::local();
        target.resolution = "host_missing".to_string();
        target.host_id = Some(placement.host_id.clone());
        return target;
    };

    match AoRemoteDaemonClient::new(host.address.clone()) {
        Ok(client) => ProjectDaemonTarget {
            controller: ProjectDaemonController::Remote(client),
            transport: "remote_http".to_string(),
            host_id: Some(host.id.clone()),
            host_slug: Some(host.slug.clone()),
            host_address: Some(host.address.clone()),
            resolution: "host_http_endpoint".to_string(),
        },
        Err(_) => ProjectDaemonTarget {
            controller: ProjectDaemonController::Local(AoDaemonClient::new()),
            transport: "local_cli".to_string(),
            host_id: Some(host.id.clone()),
            host_slug: Some(host.slug.clone()),
            host_address: Some(host.address.clone()),
            resolution: "host_address_not_http_url".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn resolves_remote_http_target_for_project_placement() {
        let project = fixture_project();
        let placement_map = build_project_host_placement_map(vec![ProjectHostPlacement {
            project_id: project.id.clone(),
            host_id: "host-1".to_string(),
            assignment_source: "founder".to_string(),
            assigned_at: Utc::now(),
        }]);
        let host_map = build_host_map(vec![fixture_host("http://host.test:7444")]);

        let target = resolve_project_daemon_target(&project, &placement_map, &host_map);

        assert_eq!(target.source_name(), "ao-web-api");
        assert_eq!(target.details()["transport"], "remote_http");
        assert_eq!(target.details()["resolution"], "host_http_endpoint");
    }

    #[test]
    fn falls_back_to_local_when_host_address_is_not_http() {
        let project = fixture_project();
        let placement_map = build_project_host_placement_map(vec![ProjectHostPlacement {
            project_id: project.id.clone(),
            host_id: "host-1".to_string(),
            assignment_source: "founder".to_string(),
            assigned_at: Utc::now(),
        }]);
        let host_map = build_host_map(vec![fixture_host("founder.local")]);

        let target = resolve_project_daemon_target(&project, &placement_map, &host_map);

        assert_eq!(target.source_name(), "ao-cli");
        assert_eq!(target.details()["transport"], "local_cli");
        assert_eq!(target.details()["resolution"], "host_address_not_http_url");
    }

    fn fixture_host(address: &str) -> Host {
        Host {
            id: "host-1".to_string(),
            slug: "founder".to_string(),
            name: "Founder".to_string(),
            address: address.to_string(),
            platform: "macos".to_string(),
            status: "healthy".to_string(),
            capacity_slots: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn fixture_project() -> Project {
        Project {
            id: "project-1".to_string(),
            team_id: "team-1".to_string(),
            slug: "app".to_string(),
            root_path: "/tmp/app".to_string(),
            ao_project_root: "/tmp/app".to_string(),
            default_branch: "main".to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
