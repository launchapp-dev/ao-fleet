use std::collections::BTreeSet;

use anyhow::Result;
use ao_fleet_ao::{AoHostdClient, HostdHostProfile};
use ao_fleet_core::{Host, NewHost, NewProject, ProjectHostPlacement};
use ao_fleet_store::FleetStore;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone)]
pub(crate) struct HostSyncOptions {
    pub auth_token: Option<String>,
    pub register_missing: bool,
    pub team_id: Option<String>,
    pub assignment_source: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct HostSyncResult {
    pub host: Host,
    pub created_host: bool,
    pub updated_host: bool,
    pub fleet_url_hint: Option<String>,
    pub discovered_project_count: usize,
    pub register_missing: bool,
    pub team_id: Option<String>,
    pub projects: Vec<SyncedProject>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncedProject {
    pub remote_project_id: String,
    pub name: String,
    pub ao_project_root: String,
    pub existing_project_id: Option<String>,
    pub registered_project_id: Option<String>,
    pub created: bool,
    pub placement_updated: bool,
}

struct HostSyncOutcome {
    host: Host,
    created: bool,
    updated: bool,
}

pub(crate) fn sync_host_by_base_url(
    store: &FleetStore,
    base_url: &str,
    options: &HostSyncOptions,
) -> Result<HostSyncResult> {
    let client = AoHostdClient::new(base_url.to_string(), options.auth_token.clone())?;
    sync_host_with_client(store, client, options)
}

fn sync_host_with_client(
    store: &FleetStore,
    client: AoHostdClient,
    options: &HostSyncOptions,
) -> Result<HostSyncResult> {
    let remote_host = client.host_profile()?;
    let remote_projects = client.list_projects()?;
    let host_sync = upsert_host(store, &remote_host)?;
    let mut used_slugs: BTreeSet<String> =
        store.list_projects(None)?.into_iter().map(|project| project.slug).collect();
    let mut synced_projects = Vec::new();

    for remote_project in remote_projects {
        let existing = store.get_project_by_ao_project_root(&remote_project.ao_project_root)?;
        let existing_project_id = existing.as_ref().map(|project| project.id.clone());
        let mut registered_project_id = existing_project_id.clone();
        let mut created = false;
        let mut placement_updated = false;

        if options.register_missing && existing.is_none() {
            let project = store.create_project(NewProject {
                team_id: options.team_id.clone().expect("team_id validated by caller"),
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

        if let Some(project_id) = &registered_project_id {
            store.upsert_project_host_placement(ProjectHostPlacement {
                project_id: project_id.clone(),
                host_id: host_sync.host.id.clone(),
                assignment_source: options.assignment_source.clone(),
                assigned_at: Utc::now(),
            })?;
            placement_updated = true;
        }

        synced_projects.push(SyncedProject {
            remote_project_id: remote_project.id,
            name: remote_project.name,
            ao_project_root: remote_project.ao_project_root,
            existing_project_id,
            registered_project_id,
            created,
            placement_updated,
        });
    }

    Ok(HostSyncResult {
        host: host_sync.host,
        created_host: host_sync.created,
        updated_host: host_sync.updated,
        fleet_url_hint: remote_host.fleet_url,
        discovered_project_count: synced_projects.len(),
        register_missing: options.register_missing,
        team_id: options.team_id.clone(),
        projects: synced_projects,
    })
}

fn upsert_host(store: &FleetStore, remote_host: &HostdHostProfile) -> Result<HostSyncOutcome> {
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
        return Ok(HostSyncOutcome { host, created: false, updated: true });
    }

    let host = store.create_host(NewHost {
        slug: remote_host.slug.clone(),
        name: remote_host.name.clone(),
        address: remote_host.address.clone(),
        platform: remote_host.platform.clone(),
        status: remote_host.status.clone(),
        capacity_slots: remote_host.capacity_slots,
    })?;
    Ok(HostSyncOutcome { host, created: true, updated: false })
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
