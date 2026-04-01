/// E2E tests for the founder bootstrap flow.
///
/// These tests call handler functions directly with a temp-file database,
/// mirroring the exact CLI command sequence a founder follows when setting up
/// ao-fleet for the first time:
///   1. db-init (implicit in FleetStore::open)
///   2. team-create
///   3. project-create
///   4. project-list
///   5. founder-overview
///   6. daemon-health-rollup
///   7. project-status
use tempfile::NamedTempFile;

use ao_fleet_cli::cli::handlers::daemon_health_rollup::daemon_health_rollup;
use ao_fleet_cli::cli::handlers::daemon_health_rollup_command::DaemonHealthRollupCommand;
use ao_fleet_cli::cli::handlers::db_init::db_init;
use ao_fleet_cli::cli::handlers::db_init_command::DbInitCommand;
use ao_fleet_cli::cli::handlers::founder_overview::founder_overview;
use ao_fleet_cli::cli::handlers::founder_overview_command::FounderOverviewCommand;
use ao_fleet_cli::cli::handlers::project_create::project_create;
use ao_fleet_cli::cli::handlers::project_create_command::ProjectCreateCommand;
use ao_fleet_cli::cli::handlers::project_list::project_list;
use ao_fleet_cli::cli::handlers::project_list_command::ProjectListCommand;
use ao_fleet_cli::cli::handlers::project_status::project_status;
use ao_fleet_cli::cli::handlers::project_status_command::ProjectStatusCommand;
use ao_fleet_cli::cli::handlers::team_create::team_create;
use ao_fleet_cli::cli::handlers::team_create_command::TeamCreateCommand;

fn tmp_db() -> NamedTempFile {
    NamedTempFile::new().expect("temp file")
}

fn db_path(f: &NamedTempFile) -> &str {
    f.path().to_str().expect("utf-8 path")
}

/// A founder bootstraps a fleet from scratch: init → team → project → list → overview.
#[test]
fn test_founder_bootstrap_full_flow() {
    let db = tmp_db();
    let path = db_path(&db);

    // Step 1: init db (no-op if already exists, idempotent)
    db_init(path, DbInitCommand).expect("db_init succeeded");

    // Step 2: create the founding team
    team_create(
        path,
        TeamCreateCommand {
            slug: "acme".to_string(),
            name: "Acme Corp".to_string(),
            mission: "Ship fast, break nothing".to_string(),
            ownership: "founder".to_string(),
            business_priority: 100,
        },
    )
    .expect("team_create succeeded");

    // Step 3: register the first project
    // We need the team id — use the store directly to look it up
    let store = ao_fleet_store::FleetStore::open(path).expect("store opens");
    let teams = store.list_teams().expect("list teams");
    assert_eq!(teams.len(), 1);
    let team_id = &teams[0].id;

    project_create(
        path,
        ProjectCreateCommand {
            team_id: team_id.clone(),
            slug: "core-api".to_string(),
            root_path: "/tmp/core-api".to_string(),
            ao_project_root: "/tmp/core-api".to_string(),
            default_branch: "main".to_string(),
            remote_url: Some("https://github.com/acme/core-api".to_string()),
            enabled: true,
        },
    )
    .expect("project_create succeeded");

    // Step 4: list projects — should see exactly one
    let projects = store.list_projects(None).expect("list projects");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].slug, "core-api");

    // Step 5: project-list command succeeds (prints JSON to stdout)
    project_list(path, ProjectListCommand).expect("project_list command succeeded");

    // Step 6: founder-overview command succeeds
    founder_overview(path, FounderOverviewCommand { team_id: None })
        .expect("founder_overview succeeded");
}

/// The daemon-health-rollup command works on an empty fleet (zero projects).
#[test]
fn test_daemon_health_rollup_empty_fleet() {
    let db = tmp_db();
    let path = db_path(&db);
    db_init(path, DbInitCommand).expect("db_init succeeded");

    daemon_health_rollup(path, DaemonHealthRollupCommand { team_id: None })
        .expect("daemon_health_rollup on empty fleet succeeded");
}

/// The daemon-health-rollup command works after projects are registered,
/// reflecting zero observed statuses (no daemons polled yet).
#[test]
fn test_daemon_health_rollup_with_projects() {
    let db = tmp_db();
    let path = db_path(&db);
    db_init(path, DbInitCommand).expect("db_init succeeded");

    let store = ao_fleet_store::FleetStore::open(path).expect("store opens");
    let team = store
        .create_team(ao_fleet_core::NewTeam {
            slug: "beta".to_string(),
            name: "Beta Team".to_string(),
            mission: "Test everything".to_string(),
            ownership: "eng".to_string(),
            business_priority: 50,
        })
        .expect("team created");

    store
        .create_project(ao_fleet_core::NewProject {
            team_id: team.id.clone(),
            slug: "beta-api".to_string(),
            root_path: "/tmp/beta-api".to_string(),
            ao_project_root: "/tmp/beta-api".to_string(),
            default_branch: "main".to_string(),
            remote_url: None,
            enabled: true,
        })
        .expect("project created");

    // health rollup: 1 total, 0 aligned (no observed statuses yet), 1 unobserved
    daemon_health_rollup(path, DaemonHealthRollupCommand { team_id: None })
        .expect("daemon_health_rollup with projects succeeded");

    // Also verify rollup filtered by team works
    daemon_health_rollup(
        path,
        DaemonHealthRollupCommand { team_id: Some(team.id.clone()) },
    )
    .expect("daemon_health_rollup with team filter succeeded");
}

/// project-status returns an error for a nonexistent project.
#[test]
fn test_project_status_not_found() {
    let db = tmp_db();
    let path = db_path(&db);
    db_init(path, DbInitCommand).expect("db_init succeeded");

    let result = project_status(
        path,
        ProjectStatusCommand { id: "nonexistent-id".to_string(), refresh: false },
    );
    assert!(result.is_err(), "expected error for missing project");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("nonexistent-id"), "error mentions project id");
}

/// project-status works for a registered project (no live refresh).
#[test]
fn test_project_status_found() {
    let db = tmp_db();
    let path = db_path(&db);
    db_init(path, DbInitCommand).expect("db_init succeeded");

    let store = ao_fleet_store::FleetStore::open(path).expect("store opens");
    let team = store
        .create_team(ao_fleet_core::NewTeam {
            slug: "gamma".to_string(),
            name: "Gamma".to_string(),
            mission: "Deploy".to_string(),
            ownership: "ops".to_string(),
            business_priority: 10,
        })
        .expect("team created");

    let project = store
        .create_project(ao_fleet_core::NewProject {
            team_id: team.id.clone(),
            slug: "gamma-worker".to_string(),
            root_path: "/tmp/gamma-worker".to_string(),
            ao_project_root: "/tmp/gamma-worker".to_string(),
            default_branch: "main".to_string(),
            remote_url: None,
            enabled: true,
        })
        .expect("project created");

    project_status(
        path,
        ProjectStatusCommand { id: project.id.clone(), refresh: false },
    )
    .expect("project_status for known project succeeded");
}
