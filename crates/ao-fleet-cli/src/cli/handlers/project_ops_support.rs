use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow, bail};
use ao_fleet_core::{Host, Project};
use ao_fleet_store::FleetStore;
use reqwest::Method;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::cli::handlers::project_events_command::ProjectEventsCommand;

pub(crate) enum ProjectOperationTarget {
    Local(Project),
    Remote { project: Project, base_url: String },
}

impl ProjectOperationTarget {
    pub(crate) fn project(&self) -> &Project {
        match self {
            Self::Local(project) => project,
            Self::Remote { project, .. } => project,
        }
    }
}

pub(crate) fn resolve_project_operation_target(
    db_path: &str,
    project_root: &str,
) -> Result<ProjectOperationTarget> {
    let store = FleetStore::open(db_path)?;
    let project = store
        .get_project_by_ao_project_root(project_root)?
        .ok_or_else(|| anyhow!("project not found in fleet registry: {project_root}"))?;

    let placements = store.list_project_host_placements()?;
    let hosts = store.list_hosts()?;
    let host_map = hosts.into_iter().map(|host| (host.id.clone(), host)).collect::<HashMap<_, _>>();

    if let Some(placement) =
        placements.into_iter().find(|placement| placement.project_id == project.id)
    {
        if let Some(host) = host_map.get(&placement.host_id) {
            if is_remote_http_host(host) {
                return Ok(ProjectOperationTarget::Remote {
                    project,
                    base_url: host.address.clone(),
                });
            }
        }
    }

    Ok(ProjectOperationTarget::Local(project))
}

pub(crate) fn execute_project_json_command(
    db_path: &str,
    project_root: &str,
    args: &[String],
) -> Result<Value> {
    let target = resolve_project_operation_target(db_path, project_root)?;
    match target {
        ProjectOperationTarget::Local(project) => run_local_ao_json(&project.ao_project_root, args),
        ProjectOperationTarget::Remote { base_url, .. } => run_remote_project_json(&base_url, args),
    }
}

pub(crate) fn load_project_config_value(db_path: &str, project_root: &str) -> Result<Value> {
    let target = resolve_project_operation_target(db_path, project_root)?;
    let project = target.project();
    let project_name = project_name_from_root(&project.ao_project_root);
    let workflow_dir = PathBuf::from(&project.ao_project_root).join(".ao").join("workflows");
    let mut merged = Map::new();

    let entries = std::fs::read_dir(&workflow_dir).map_err(|error| {
        anyhow!("failed to read workflow directory {}: {error}", workflow_dir.display())
    })?;

    let mut files = entries.filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.file_name());

    for entry in files {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|error| anyhow!("failed to read workflow file {}: {error}", path.display()))?;
        let value = serde_yaml::from_str::<Value>(&content)
            .map_err(|error| anyhow!("invalid yaml {}: {error}", path.display()))?;
        merge_config_value(&mut merged, value);
    }

    let agents = merged
        .get("agents")
        .and_then(Value::as_object)
        .map(|agents| {
            agents
                .iter()
                .map(|(name, value)| {
                    json!({
                        "name": name,
                        "model": value["model"].as_str().unwrap_or("default"),
                        "tool": value["tool"].as_str().unwrap_or("claude"),
                        "system_prompt": value["system_prompt"].as_str(),
                        "mcp_servers": value["mcp_servers"].as_array().cloned().unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let phases = merged
        .get("phases")
        .and_then(Value::as_object)
        .map(|phases| {
            phases
                .iter()
                .map(|(id, value)| {
                    json!({
                        "id": id,
                        "mode": value["mode"].as_str().unwrap_or("agent"),
                        "agent": value["agent"].as_str(),
                        "directive": value["directive"].as_str(),
                        "command": value["command"]["program"].as_str(),
                        "command_args": value["command"]["args"].as_array().cloned().unwrap_or_default(),
                        "timeout_secs": value["command"]["timeout_secs"].as_i64().or_else(|| value["timeout_secs"].as_i64()),
                        "cwd_mode": value["command"]["cwd_mode"].as_str(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let workflows = merged
        .get("workflows")
        .and_then(Value::as_array)
        .map(|workflows| {
            let mut seen = std::collections::HashSet::new();
            workflows
                .iter()
                .filter_map(|workflow| {
                    let id = workflow["id"].as_str()?.to_string();
                    if !seen.insert(id.clone()) {
                        return None;
                    }
                    let phases = workflow["phases"]
                        .as_array()
                        .map(|phases| {
                            phases
                                .iter()
                                .filter_map(|phase| {
                                    phase.as_str().map(ToOwned::to_owned).or_else(|| {
                                        phase["phase_ref"].as_str().map(ToOwned::to_owned)
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    Some(json!({
                        "id": id,
                        "name": workflow["name"].as_str(),
                        "description": workflow["description"].as_str(),
                        "phases": phases,
                    }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let schedules = merged
        .get("schedules")
        .and_then(Value::as_array)
        .map(|schedules| {
            let mut seen = std::collections::HashSet::new();
            schedules
                .iter()
                .filter_map(|schedule| {
                    let id = schedule["id"].as_str()?.to_string();
                    if !seen.insert(id.clone()) {
                        return None;
                    }
                    Some(json!({
                        "id": id,
                        "cron": schedule["cron"].as_str().unwrap_or(""),
                        "workflow_ref": schedule["workflow_ref"].as_str().unwrap_or(""),
                        "enabled": schedule["enabled"].as_bool().unwrap_or(true),
                    }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(json!({
        "project": project_name,
        "root": project.ao_project_root,
        "agents": agents,
        "phases": phases,
        "workflows": workflows,
        "schedules": schedules,
    }))
}

pub(crate) fn list_project_events(
    db_path: &str,
    command: &ProjectEventsCommand,
) -> Result<Vec<Value>> {
    let target = resolve_project_operation_target(db_path, &command.project_root)?;
    match target {
        ProjectOperationTarget::Local(project) => read_local_project_events(
            &project.ao_project_root,
            command.workflow.as_deref(),
            command.run.as_deref(),
            command.cat.as_deref(),
            command.level.as_deref(),
            command.tail,
        ),
        ProjectOperationTarget::Remote { .. } => {
            bail!("project-events currently supports local fleet projects only")
        }
    }
}

pub(crate) fn stream_project_events(db_path: &str, command: &ProjectEventsCommand) -> Result<()> {
    let target = resolve_project_operation_target(db_path, &command.project_root)?;
    match target {
        ProjectOperationTarget::Local(project) => follow_local_project_events(
            &project.ao_project_root,
            command.workflow.as_deref(),
            command.run.as_deref(),
            command.cat.as_deref(),
            command.level.as_deref(),
            command.tail,
        ),
        ProjectOperationTarget::Remote { .. } => {
            bail!("project-events --follow currently supports local fleet projects only")
        }
    }
}

fn is_remote_http_host(host: &Host) -> bool {
    host.address.starts_with("http://") || host.address.starts_with("https://")
}

fn run_local_ao_json(project_root: &str, args: &[String]) -> Result<Value> {
    let mut full_args = args.iter().map(OsString::from).collect::<Vec<_>>();
    if !full_args.iter().any(|arg| arg == "--json") {
        full_args.push(OsString::from("--json"));
    }
    if !full_args.iter().any(|arg| arg == "--project-root") {
        full_args.push(OsString::from("--project-root"));
        full_args.push(OsString::from(project_root));
    }

    let output = Command::new(resolve_ao_binary())
        .args(&full_args)
        .output()
        .map_err(|error| anyhow!("failed to run ao: {error}"))?;

    if !output.status.success() {
        bail!("ao command failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }

    parse_local_cli_envelope(&output.stdout)
}

fn parse_local_cli_envelope(stdout: &[u8]) -> Result<Value> {
    let parsed = serde_json::from_slice::<LocalAoEnvelope>(stdout)
        .map_err(|error| anyhow!("invalid ao json envelope: {error}"))?;
    if !parsed.ok {
        bail!("ao command reported ok=false");
    }
    Ok(parsed.data)
}

fn run_remote_project_json(base_url: &str, args: &[String]) -> Result<Value> {
    if args.len() < 2 {
        bail!("project-ao-json requires at least a command group and operation");
    }

    let client = Client::new();
    match (args[0].as_str(), args[1].as_str()) {
        ("daemon", "status") => {
            send_remote_request(&client, Method::GET, base_url, "/daemon/status", &[], None)
        }
        ("daemon", "start") => {
            send_remote_request(&client, Method::POST, base_url, "/daemon/start", &[], None)
        }
        ("daemon", "stop") => {
            send_remote_request(&client, Method::POST, base_url, "/daemon/stop", &[], None)
        }
        ("daemon", "pause") => {
            send_remote_request(&client, Method::POST, base_url, "/daemon/pause", &[], None)
        }
        ("daemon", "resume") => {
            send_remote_request(&client, Method::POST, base_url, "/daemon/resume", &[], None)
        }
        ("workflow", "list") => {
            let mut query = Vec::<(String, String)>::new();
            if let Some(status) = flag_value(args, "--status") {
                query.push(("status".to_string(), status));
            }
            if let Some(search) = flag_value(args, "--search") {
                query.push(("search".to_string(), search));
            }
            normalize_collection_value(send_remote_request(
                &client,
                Method::GET,
                base_url,
                "/workflows",
                &query,
                None,
            )?)
        }
        ("task", "list") => {
            let offset = flag_value(args, "--offset")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = flag_value(args, "--limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100);
            let mut query = Vec::<(String, String)>::new();
            if let Some(status) = flag_value(args, "--status") {
                query.push(("status".to_string(), status));
            }
            if let Some(priority) = flag_value(args, "--priority") {
                query.push(("priority".to_string(), priority));
            }
            if let Some(search) = flag_value(args, "--search") {
                query.push(("search".to_string(), search));
            }
            query.push(("page_size".to_string(), (offset + limit).to_string()));
            let data = normalize_collection_value(send_remote_request(
                &client,
                Method::GET,
                base_url,
                "/tasks",
                &query,
                None,
            )?)?;
            Ok(slice_array_value(data, offset, limit))
        }
        ("task", "prioritized") => {
            let mut query = Vec::<(String, String)>::new();
            if let Some(status) = flag_value(args, "--status") {
                query.push(("status".to_string(), status));
            }
            if let Some(priority) = flag_value(args, "--priority") {
                query.push(("priority".to_string(), priority));
            }
            if let Some(search) = flag_value(args, "--search") {
                query.push(("search".to_string(), search));
            }
            normalize_collection_value(send_remote_request(
                &client,
                Method::GET,
                base_url,
                "/tasks/prioritized",
                &query,
                None,
            )?)
        }
        ("task", "get") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(&client, Method::GET, base_url, &format!("/tasks/{id}"), &[], None)
        }
        ("task", "stats") => {
            send_remote_request(&client, Method::GET, base_url, "/tasks/stats", &[], None)
        }
        ("task", "next") => {
            send_remote_request(&client, Method::GET, base_url, "/tasks/next", &[], None)
        }
        ("task", "create") => send_remote_request(
            &client,
            Method::POST,
            base_url,
            "/tasks",
            &[],
            Some(json!({
                "title": required_flag_value(args, "--title")?,
                "description": flag_value(args, "--description"),
                "task_type": flag_value(args, "--task-type"),
                "priority": flag_value(args, "--priority"),
            })),
        ),
        ("task", "update") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(
                &client,
                Method::PATCH,
                base_url,
                &format!("/tasks/{id}"),
                &[],
                Some(json!({
                    "title": flag_value(args, "--title"),
                    "description": flag_value(args, "--description"),
                    "priority": flag_value(args, "--priority"),
                    "status": flag_value(args, "--status"),
                    "assignee": flag_value(args, "--assignee"),
                })),
            )
        }
        ("task", "status") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(
                &client,
                Method::POST,
                base_url,
                &format!("/tasks/{id}/status"),
                &[],
                Some(json!({
                    "status": required_flag_value(args, "--status")?,
                })),
            )
        }
        ("task", "assign") => {
            let id = required_flag_value(args, "--id")?;
            let assignee = required_flag_value(args, "--assignee")?;
            let assignee_type = flag_value(args, "--type").unwrap_or_else(|| "human".to_string());
            if assignee_type == "agent" {
                send_remote_request(
                    &client,
                    Method::POST,
                    base_url,
                    &format!("/tasks/{id}/assign-agent"),
                    &[],
                    Some(json!({
                        "role": flag_value(args, "--agent-role").unwrap_or(assignee),
                        "model": flag_value(args, "--model"),
                    })),
                )
            } else {
                send_remote_request(
                    &client,
                    Method::POST,
                    base_url,
                    &format!("/tasks/{id}/assign-human"),
                    &[],
                    Some(json!({
                        "user_id": assignee,
                    })),
                )
            }
        }
        ("task", "set-priority") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(
                &client,
                Method::PATCH,
                base_url,
                &format!("/tasks/{id}"),
                &[],
                Some(json!({
                    "priority": required_flag_value(args, "--priority")?,
                })),
            )
        }
        ("task", "set-deadline") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(
                &client,
                Method::PATCH,
                base_url,
                &format!("/tasks/{id}"),
                &[],
                Some(json!({
                    "deadline": flag_value(args, "--deadline"),
                })),
            )
        }
        ("task", "checklist-add") => {
            let id = required_flag_value(args, "--id")?;
            send_remote_request(
                &client,
                Method::POST,
                base_url,
                &format!("/tasks/{id}/checklist"),
                &[],
                Some(json!({
                    "description": required_flag_value(args, "--description")?,
                })),
            )
        }
        ("task", "checklist-update") => {
            let id = required_flag_value(args, "--id")?;
            let item_id = required_flag_value(args, "--item-id")?;
            send_remote_request(
                &client,
                Method::PATCH,
                base_url,
                &format!("/tasks/{id}/checklist/{item_id}"),
                &[],
                Some(json!({
                    "completed": required_flag_value(args, "--completed")? == "true",
                })),
            )
        }
        _ => bail!("remote project operation is not supported yet: {}", args.join(" ")),
    }
}

fn send_remote_request(
    client: &Client,
    method: Method,
    base_url: &str,
    path: &str,
    query: &[(String, String)],
    body: Option<Value>,
) -> Result<Value> {
    let url = format!("{}/api/v1{}", base_url.trim_end_matches('/'), path);
    let mut request = client.request(method, &url).query(query);
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().map_err(|error| anyhow!("{url}: {error}"))?;
    let status = response.status();
    let text = response.text().map_err(|error| anyhow!("{url}: {error}"))?;
    let envelope = serde_json::from_str::<RemoteAoEnvelope>(&text)
        .map_err(|error| anyhow!("{url}: invalid json: {error}"))?;

    if !status.is_success() {
        bail!("{url}: http {status}: {}", remote_error_message(&envelope));
    }

    if !envelope.ok {
        bail!("{url}: remote api reported ok=false: {}", remote_error_message(&envelope));
    }

    Ok(envelope.data.unwrap_or(Value::Null))
}

fn normalize_collection_value(value: Value) -> Result<Value> {
    if value.is_array() {
        return Ok(value);
    }

    if let Some(items) = value.get("items").and_then(Value::as_array) {
        return Ok(Value::Array(items.clone()));
    }

    bail!("expected array-like response")
}

fn slice_array_value(value: Value, offset: usize, limit: usize) -> Value {
    let Some(items) = value.as_array() else {
        return value;
    };

    let sliced = items.iter().skip(offset).take(limit).cloned().collect::<Vec<_>>();
    Value::Array(sliced)
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|window| window[0] == flag).map(|window| window[1].clone())
}

fn required_flag_value(args: &[String], flag: &str) -> Result<String> {
    flag_value(args, flag).ok_or_else(|| anyhow!("missing required flag: {flag}"))
}

fn remote_error_message(envelope: &RemoteAoEnvelope) -> String {
    envelope
        .error
        .as_ref()
        .and_then(|value| value.get("message").and_then(Value::as_str).map(ToOwned::to_owned))
        .or_else(|| envelope.error.as_ref().map(Value::to_string))
        .unwrap_or_else(|| "unknown remote error".to_string())
}

fn resolve_ao_binary() -> PathBuf {
    if let Some(explicit) = std::env::var_os("AO_BIN") {
        return PathBuf::from(explicit);
    }

    if let Some(home) = std::env::var_os("HOME") {
        let local_binary = PathBuf::from(home).join(".local").join("bin").join("ao");
        if local_binary.exists() {
            return local_binary;
        }
    }

    PathBuf::from("ao")
}

fn read_local_project_events(
    project_root: &str,
    workflow: Option<&str>,
    run: Option<&str>,
    cat: Option<&str>,
    level: Option<&str>,
    tail: usize,
) -> Result<Vec<Value>> {
    let output = Command::new(resolve_ao_binary())
        .args(build_local_stream_args(project_root, workflow, run, cat, level, tail, false))
        .output()
        .map_err(|error| anyhow!("failed to read project events: {error}"))?;

    if !output.status.success() {
        bail!("failed to read project events: {}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let project_name = project_name_from_root(project_root);
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .map(|value| normalize_local_stream_event(&project_name, project_root, &value))
        .collect())
}

fn follow_local_project_events(
    project_root: &str,
    workflow: Option<&str>,
    run: Option<&str>,
    cat: Option<&str>,
    level: Option<&str>,
    tail: usize,
) -> Result<()> {
    let mut child = Command::new(resolve_ao_binary())
        .args(build_local_stream_args(project_root, workflow, run, cat, level, tail, true))
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| anyhow!("failed to start project event stream: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("project event stream stdout was unavailable"))?;
    let reader = BufReader::new(stdout);
    let project_name = project_name_from_root(project_root);
    let mut handle = std::io::stdout().lock();

    for line in reader.lines() {
        let line = line?;
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let normalized = normalize_local_stream_event(&project_name, project_root, &value);
        writeln!(handle, "{}", serde_json::to_string(&normalized)?)?;
        handle.flush()?;
    }

    let status =
        child.wait().map_err(|error| anyhow!("failed to wait for event stream: {error}"))?;
    if !status.success() {
        bail!("project event stream exited unsuccessfully");
    }

    Ok(())
}

fn build_local_stream_args(
    project_root: &str,
    workflow: Option<&str>,
    run: Option<&str>,
    cat: Option<&str>,
    level: Option<&str>,
    tail: usize,
    follow: bool,
) -> Vec<String> {
    let mut args = vec![
        "daemon".to_string(),
        "stream".to_string(),
        "--project-root".to_string(),
        project_root.to_string(),
        "--json".to_string(),
        "--tail".to_string(),
        tail.to_string(),
    ];

    if let Some(value) = workflow.filter(|value| !value.is_empty()) {
        args.push("--workflow".to_string());
        args.push(value.to_string());
    } else if let Some(value) = run.filter(|value| !value.is_empty()) {
        args.push("--run".to_string());
        args.push(value.to_string());
    }

    if let Some(value) = cat.filter(|value| !value.is_empty()) {
        args.push("--cat".to_string());
        args.push(value.to_string());
    }

    if let Some(value) = level.filter(|value| !value.is_empty() && *value != "all") {
        args.push("--level".to_string());
        args.push(value.to_string());
    }

    if !follow {
        args.push("--no-follow".to_string());
    }

    args
}

fn normalize_local_stream_event(project_name: &str, project_root: &str, value: &Value) -> Value {
    let run_id = value["run_id"].as_str().map(ToOwned::to_owned);
    let workflow_id = run_id.as_deref().and_then(parse_workflow_id_from_run_id);

    json!({
        "project": project_name,
        "project_root": project_root,
        "ts": value["ts"].as_str().unwrap_or(""),
        "level": value["level"].as_str().unwrap_or("info"),
        "cat": value["cat"].as_str().unwrap_or(""),
        "msg": value["msg"].as_str().unwrap_or(""),
        "role": value["role"].as_str(),
        "content": value["content"].as_str(),
        "error": value["error"].as_str(),
        "run_id": run_id,
        "workflow_id": workflow_id,
        "subject_id": value["subject_id"].as_str(),
        "phase_id": value["phase_id"].as_str(),
        "task_id": value["task_id"].as_str(),
        "workflow_ref": value["meta"]["workflow_ref"].as_str().or_else(|| value["workflow_ref"].as_str()),
        "model": value["model"].as_str(),
        "tool": value["meta"]["tool"].as_str().or_else(|| value["tool"].as_str()),
        "schedule_id": value["schedule_id"].as_str(),
        "meta": value.get("meta").cloned(),
    })
}

fn parse_workflow_id_from_run_id(run_id: &str) -> Option<String> {
    let trimmed = run_id.strip_prefix("wf-")?;
    if trimmed.len() < 36 {
        return None;
    }

    let candidate = &trimmed[..36];
    let is_uuid = candidate.bytes().enumerate().all(|(index, byte)| match index {
        8 | 13 | 18 | 23 => byte == b'-',
        _ => byte.is_ascii_hexdigit(),
    });

    is_uuid.then(|| candidate.to_string())
}

fn project_name_from_root(project_root: &str) -> String {
    PathBuf::from(project_root)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn merge_config_value(merged: &mut Map<String, Value>, value: Value) {
    let Some(object) = value.as_object() else {
        return;
    };

    for (key, next_value) in object {
        match (merged.get(key.as_str()), next_value) {
            (Some(Value::Object(existing)), Value::Object(next_object)) => {
                let mut combined = existing.clone();
                for (entry_key, entry_value) in next_object {
                    combined.insert(entry_key.clone(), entry_value.clone());
                }
                merged.insert(key.clone(), Value::Object(combined));
            }
            (Some(Value::Array(existing)), Value::Array(next_array)) => {
                let mut combined = existing.clone();
                combined.extend(next_array.iter().cloned());
                merged.insert(key.clone(), Value::Array(combined));
            }
            _ => {
                merged.insert(key.clone(), next_value.clone());
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct LocalAoEnvelope {
    ok: bool,
    data: Value,
}

#[derive(Debug, Deserialize)]
struct RemoteAoEnvelope {
    ok: bool,
    data: Option<Value>,
    error: Option<Value>,
}
