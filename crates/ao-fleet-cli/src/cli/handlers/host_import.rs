use std::collections::BTreeSet;

use anyhow::{Result, bail};
use ao_fleet_ao::{AoHostdClient, HostdHostProfile};
use ao_fleet_core::{Host, NewHost, NewProject, ProjectHostPlacement};
use ao_fleet_store::FleetStore;
use chrono::Utc;
use serde::Serialize;

use crate::cli::handlers::host_import_command::HostImportCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_import(db_path: &str, command: HostImportCommand) -> Result<()> {
    if command.register_projects && command.team_id.is_none() {
        bail!("--team-id is required when --register-projects is set");
    }

    let client = AoHostdClient::new(command.base_url.clone(), command.auth_token.clone())?;
    let remote_host = client.host_profile()?;
    let remote_projects = client.list_projects()?;
    let store = FleetStore::open(db_path)?;

    let host_import = upsert_host(&store, &remote_host)?;
    let team_id = command.team_id.clone();
    let mut used_slugs: BTreeSet<String> =
        store.list_projects(None)?.into_iter().map(|project| project.slug).collect();
    let mut imported_projects = Vec::new();

    for remote_project in remote_projects {
        let existing = store.get_project_by_ao_project_root(&remote_project.ao_project_root)?;
        let existing_project_id = existing.as_ref().map(|project| project.id.clone());
        let mut registered_project_id = existing_project_id.clone();
        let mut placement_updated = false;
        let mut created = false;

        if command.register_projects && existing.is_none() {
            let project = store.create_project(NewProject {
                team_id: team_id.clone().expect("team_id validated"),
                slug: allocate_unique_slug(slugify(&remote_project.name), &mut used_slugs),
                root_path: remote_project.root_path.clone(),
                ao_project_root: remote_project.ao_project_root.clone(),
                default_branch: remote_project.default_branch.clone(),
                remote_url: remote_project.remote_url.clone(),
                enabled: remote_project.enabled,
            })?;
            registered_project_id = Some(project.id);
            created = true;
        }

        if command.register_projects {
            if let Some(project_id) = &registered_project_id {
                store.upsert_project_host_placement(ProjectHostPlacement {
                    project_id: project_id.clone(),
                    host_id: host_import.host.id.clone(),
                    assignment_source: command.assignment_source.clone(),
                    assigned_at: Utc::now(),
                })?;
                placement_updated = true;
            }
        }

        imported_projects.push(ImportedProject {
            remote_project_id: remote_project.id,
            name: remote_project.name,
            ao_project_root: remote_project.ao_project_root,
            existing_project_id,
            registered_project_id,
            created,
            placement_updated,
        });
    }

    print_json(&HostImportResult {
        host: host_import.host,
        created_host: host_import.created,
        updated_host: host_import.updated,
        fleet_url_hint: remote_host.fleet_url,
        discovered_project_count: imported_projects.len(),
        register_projects: command.register_projects,
        team_id: command.team_id,
        projects: imported_projects,
    })
}

#[derive(Debug, Serialize)]
struct HostImportResult {
    host: Host,
    created_host: bool,
    updated_host: bool,
    fleet_url_hint: Option<String>,
    discovered_project_count: usize,
    register_projects: bool,
    team_id: Option<String>,
    projects: Vec<ImportedProject>,
}

#[derive(Debug, Serialize)]
struct ImportedProject {
    remote_project_id: String,
    name: String,
    ao_project_root: String,
    existing_project_id: Option<String>,
    registered_project_id: Option<String>,
    created: bool,
    placement_updated: bool,
}

struct HostImportOutcome {
    host: Host,
    created: bool,
    updated: bool,
}

fn upsert_host(store: &FleetStore, remote_host: &HostdHostProfile) -> Result<HostImportOutcome> {
    let existing = store
        .list_hosts()?
        .into_iter()
        .find(|host| host.address == remote_host.address || host.slug == remote_host.slug);

    if let Some(mut host) = existing {
        host.slug = remote_host.slug.clone();
        host.name = remote_host.name.clone();
        host.address = remote_host.address.clone();
        host.platform = remote_host.platform.clone();
        host.status = remote_host.status.clone();
        host.capacity_slots = remote_host.capacity_slots;
        host.updated_at = Utc::now();
        let host = store.update_host(host)?;
        return Ok(HostImportOutcome { host, created: false, updated: true });
    }

    let host = store.create_host(NewHost {
        slug: remote_host.slug.clone(),
        name: remote_host.name.clone(),
        address: remote_host.address.clone(),
        platform: remote_host.platform.clone(),
        status: remote_host.status.clone(),
        capacity_slots: remote_host.capacity_slots,
    })?;
    Ok(HostImportOutcome { host, created: true, updated: false })
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() { "project".to_string() } else { trimmed.to_string() }
}

fn allocate_unique_slug(base_slug: String, used_slugs: &mut BTreeSet<String>) -> String {
    if used_slugs.insert(base_slug.clone()) {
        return base_slug;
    }

    for index in 2.. {
        let candidate = format!("{base_slug}-{index}");
        if used_slugs.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("slug allocation should always return")
}
