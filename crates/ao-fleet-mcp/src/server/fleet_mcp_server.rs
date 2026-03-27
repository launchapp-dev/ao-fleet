use std::io::{BufRead, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::api::fleet_mcp_api::FleetMcpApi;
use crate::error::fleet_mcp_error::FleetMcpError;
use crate::surface::fleet_mcp_surface::FleetMcpSurface;

pub struct FleetMcpServer<H> {
    handlers: H,
    surface: FleetMcpSurface,
}

impl<H> FleetMcpServer<H> {
    pub fn new(handlers: H) -> Self {
        Self { handlers, surface: FleetMcpSurface::default() }
    }

    pub fn with_surface(handlers: H, surface: FleetMcpSurface) -> Self {
        Self { handlers, surface }
    }

    pub fn surface(&self) -> &FleetMcpSurface {
        &self.surface
    }
}

impl<H: FleetMcpApi> FleetMcpServer<H> {
    pub fn handle_message(&self, message: &str) -> Result<Option<String>, FleetMcpError> {
        let request = match serde_json::from_str::<JsonRpcRequest>(message) {
            Ok(request) => request,
            Err(error) => {
                return Ok(Some(serialize_response(&JsonRpcResponse::error(
                    Value::Null,
                    FleetMcpError::InvalidRequest(format!("invalid JSON-RPC request: {error}")),
                ))?));
            }
        };

        let response = self.handle_request(request)?;
        match response {
            Some(response) => Ok(Some(serialize_response(&response)?)),
            None => Ok(None),
        }
    }

    pub fn serve_stdio(self) -> Result<(), FleetMcpError> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        while let Some(message) = read_framed_message(&mut reader)? {
            if let Some(response) = self.handle_message(&message)? {
                write_framed_message(&mut writer, &response)?;
            }
        }

        Ok(())
    }

    fn handle_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>, FleetMcpError> {
        if request.method == "notifications/initialized" {
            return Ok(None);
        }

        let id = request.id.unwrap_or(Value::Null);
        let response = match request.method.as_str() {
            "initialize" => JsonRpcResponse::result(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "ao-fleet",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }),
            ),
            "ping" => JsonRpcResponse::result(id, json!({})),
            "tools/list" => JsonRpcResponse::result(id, json!({ "tools": self.surface.tools })),
            "tools/call" => {
                let params = request
                    .params
                    .ok_or_else(|| FleetMcpError::InvalidRequest("missing params".to_string()))?;
                let call: ToolCallRequest = serde_json::from_value(params)?;
                let result = self.handle_tool_call(call)?;
                JsonRpcResponse::result(id, result)
            }
            other => {
                return Ok(Some(JsonRpcResponse::error(
                    id,
                    FleetMcpError::UnknownTool(other.to_string()),
                )));
            }
        };

        Ok(Some(response))
    }

    fn handle_tool_call(&self, call: ToolCallRequest) -> Result<Value, FleetMcpError> {
        match call.name.as_str() {
            "fleet.overview" => self.to_tool_result(
                self.handlers.fleet_overview(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.knowledge.source.list" => self.to_tool_result(
                self.handlers.list_knowledge_sources(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.knowledge.source.upsert" => self.to_tool_result(
                self.handlers.upsert_knowledge_source(parse_required_input(call.arguments)?)?,
            ),
            "fleet.knowledge.document.list" => self.to_tool_result(
                self.handlers.list_knowledge_documents(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.knowledge.document.create" => self.to_tool_result(
                self.handlers.create_knowledge_document(parse_required_input(call.arguments)?)?,
            ),
            "fleet.knowledge.fact.list" => self.to_tool_result(
                self.handlers.list_knowledge_facts(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.knowledge.fact.create" => self.to_tool_result(
                self.handlers.create_knowledge_fact(parse_required_input(call.arguments)?)?,
            ),
            "fleet.team.list" => self
                .to_tool_result(self.handlers.list_teams(parse_optional_input(call.arguments)?)?),
            "fleet.team.create" => self
                .to_tool_result(self.handlers.create_team(parse_required_input(call.arguments)?)?),
            "fleet.project.list" => self.to_tool_result(
                self.handlers.list_projects(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.project.create" => self.to_tool_result(
                self.handlers.create_project(parse_required_input(call.arguments)?)?,
            ),
            "fleet.schedule.list" => self.to_tool_result(
                self.handlers.list_schedules(parse_optional_input(call.arguments)?)?,
            ),
            "fleet.schedule.create" => self.to_tool_result(
                self.handlers.create_schedule(parse_required_input(call.arguments)?)?,
            ),
            "fleet.daemon.reconcile" => self.to_tool_result(
                self.handlers.reconcile_daemons(parse_optional_input(call.arguments)?)?,
            ),
            other => Err(FleetMcpError::UnknownTool(other.to_string())),
        }
    }

    fn to_tool_result<T: Serialize>(&self, value: T) -> Result<Value, FleetMcpError> {
        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&value)?
                }
            ]
        }))
    }
}

fn parse_required_input<T>(arguments: Option<Value>) -> Result<T, FleetMcpError>
where
    T: DeserializeOwned,
{
    let value = arguments.ok_or_else(|| {
        FleetMcpError::InvalidRequest("missing arguments for tool call".to_string())
    })?;
    Ok(serde_json::from_value(value)?)
}

fn parse_optional_input<T>(arguments: Option<Value>) -> Result<T, FleetMcpError>
where
    T: DeserializeOwned + Default,
{
    match arguments {
        Some(value) => Ok(serde_json::from_value(value)?),
        None => Ok(T::default()),
    }
}

fn read_framed_message(reader: &mut impl BufRead) -> Result<Option<String>, FleetMcpError> {
    let mut content_length = None;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = Some(value.trim().parse::<usize>().map_err(|error| {
                    FleetMcpError::InvalidRequest(format!("invalid Content-Length header: {error}"))
                })?);
            }
        }
    }

    let Some(content_length) = content_length else {
        return Err(FleetMcpError::InvalidRequest("missing Content-Length header".to_string()));
    };

    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)?;

    let body = String::from_utf8(body)
        .map_err(|error| FleetMcpError::InvalidRequest(format!("invalid UTF-8 body: {error}")))?;
    Ok(Some(body))
}

fn write_framed_message(writer: &mut impl Write, body: &str) -> Result<(), FleetMcpError> {
    let payload = body.as_bytes();
    write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

fn serialize_response(response: &JsonRpcResponse) -> Result<String, FleetMcpError> {
    Ok(serde_json::to_string(response)?)
}

#[derive(Debug, Clone, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    fn result(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }

    fn error(id: Value, error: FleetMcpError) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code: error.code(),
                message: error.to_string(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    _jsonrpc: Option<String>,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ToolCallRequest {
    name: String,
    #[serde(default)]
    arguments: Option<Value>,
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use ao_fleet_core::{
        DaemonDesiredState, KnowledgeDocument, KnowledgeDocumentKind, KnowledgeFact,
        KnowledgeFactKind, KnowledgeScope, KnowledgeSource, KnowledgeSourceKind,
        KnowledgeSyncState, Project, Schedule, SchedulePolicyKind, Team, WeekdayWindow,
    };
    use chrono::Utc;

    use crate::FleetOverview;
    use crate::FleetOverviewQuery;
    use crate::FleetOverviewSummary;
    use crate::FleetReconcileAction;
    use crate::FleetReconcilePreview;
    use crate::FleetReconcilePreviewItem;
    use crate::FleetTeamOverview;
    use crate::FleetTeamSummary;
    use crate::api::fleet_mcp_api::FleetMcpApi;
    use crate::error::fleet_mcp_error::FleetMcpError;
    use crate::inputs::daemon_reconcile_input::DaemonReconcileInput;
    use crate::inputs::knowledge_document_create_input::KnowledgeDocumentCreateInput;
    use crate::inputs::knowledge_fact_create_input::KnowledgeFactCreateInput;
    use crate::inputs::knowledge_record_list_input::KnowledgeRecordListInput;
    use crate::inputs::knowledge_source_upsert_input::KnowledgeSourceUpsertInput;
    use crate::inputs::project_create_input::ProjectCreateInput;
    use crate::inputs::project_list_input::ProjectListInput;
    use crate::inputs::schedule_create_input::ScheduleCreateInput;
    use crate::inputs::schedule_list_input::ScheduleListInput;
    use crate::inputs::team_create_input::TeamCreateInput;
    use crate::inputs::team_list_input::TeamListInput;
    use crate::results::daemon_reconcile_decision::DaemonReconcileDecision;
    use crate::results::daemon_reconcile_result::DaemonReconcileResult;

    use super::FleetMcpServer;

    #[derive(Default)]
    struct MockHandlers {
        calls: RefCell<Vec<String>>,
    }

    impl FleetMcpApi for MockHandlers {
        fn fleet_overview(
            &self,
            input: FleetOverviewQuery,
        ) -> Result<FleetOverview, FleetMcpError> {
            self.calls.borrow_mut().push("fleet_overview".to_string());
            let FleetOverviewQuery { at, backlog_by_team, .. } = input;
            let evaluated_at = at.unwrap_or_else(Utc::now);
            let backlog_count = backlog_by_team.get("team-1").copied().unwrap_or_default();

            Ok(FleetOverview {
                evaluated_at: evaluated_at.clone(),
                summary: FleetOverviewSummary {
                    team_count: 1,
                    project_count: 1,
                    schedule_count: 1,
                    enabled_project_count: 1,
                    enabled_schedule_count: 1,
                },
                teams: vec![FleetTeamOverview {
                    team: Team {
                        id: "team-1".to_string(),
                        slug: "marketing".to_string(),
                        name: "Marketing".to_string(),
                        mission: "Own campaigns".to_string(),
                        ownership: "launchapp.dev".to_string(),
                        business_priority: 10,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    },
                    summary: FleetTeamSummary {
                        project_count: 1,
                        enabled_project_count: 1,
                        schedule_count: 1,
                        enabled_schedule_count: 1,
                        backlog_count,
                    },
                    projects: vec![Project {
                        id: "project-1".to_string(),
                        team_id: "team-1".to_string(),
                        slug: "launch-site".to_string(),
                        root_path: "/tmp/launch-site".to_string(),
                        ao_project_root: "/tmp/launch-site".to_string(),
                        default_branch: "main".to_string(),
                        enabled: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }],
                    schedules: vec![Schedule {
                        id: "schedule-1".to_string(),
                        team_id: "team-1".to_string(),
                        timezone: "UTC".to_string(),
                        policy_kind: SchedulePolicyKind::AlwaysOn,
                        windows: vec![WeekdayWindow {
                            weekdays: vec![0, 1, 2, 3, 4],
                            start_hour: 9,
                            end_hour: 17,
                        }],
                        enabled: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }],
                    reconcile_preview: FleetReconcilePreviewItem {
                        team_id: "team-1".to_string(),
                        team_slug: "marketing".to_string(),
                        desired_state: DaemonDesiredState::Running,
                        observed_state: DaemonDesiredState::Paused,
                        action: FleetReconcileAction::Resume,
                        backlog_count,
                        schedule_ids: vec!["schedule-1".to_string()],
                    },
                }],
                preview: FleetReconcilePreview {
                    evaluated_at,
                    items: vec![FleetReconcilePreviewItem {
                        team_id: "team-1".to_string(),
                        team_slug: "marketing".to_string(),
                        desired_state: DaemonDesiredState::Running,
                        observed_state: DaemonDesiredState::Paused,
                        action: FleetReconcileAction::Resume,
                        backlog_count,
                        schedule_ids: vec!["schedule-1".to_string()],
                    }],
                },
            })
        }

        fn list_teams(&self, _input: TeamListInput) -> Result<Vec<Team>, FleetMcpError> {
            self.calls.borrow_mut().push("list_teams".to_string());
            Ok(vec![Team {
                id: "team-1".to_string(),
                slug: "marketing".to_string(),
                name: "Marketing".to_string(),
                mission: "Own campaigns".to_string(),
                ownership: "launchapp.dev".to_string(),
                business_priority: 10,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        }

        fn create_team(&self, input: TeamCreateInput) -> Result<Team, FleetMcpError> {
            self.calls.borrow_mut().push(format!("create_team:{}", input.slug));
            Ok(Team {
                id: "team-2".to_string(),
                slug: input.slug,
                name: input.name,
                mission: input.mission,
                ownership: input.ownership,
                business_priority: input.business_priority,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn list_knowledge_sources(
            &self,
            _input: KnowledgeRecordListInput,
        ) -> Result<Vec<KnowledgeSource>, FleetMcpError> {
            self.calls.borrow_mut().push("list_knowledge_sources".to_string());
            Ok(vec![KnowledgeSource {
                id: "source-1".to_string(),
                kind: KnowledgeSourceKind::ManualNote,
                label: "Operator note".to_string(),
                uri: None,
                scope: KnowledgeScope::Team,
                scope_ref: Some("team-1".to_string()),
                sync_state: KnowledgeSyncState::Ready,
                last_synced_at: Some(Utc::now()),
                metadata: serde_json::json!({"source": "test"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        }

        fn list_knowledge_documents(
            &self,
            _input: KnowledgeRecordListInput,
        ) -> Result<Vec<KnowledgeDocument>, FleetMcpError> {
            self.calls.borrow_mut().push("list_knowledge_documents".to_string());
            Ok(vec![KnowledgeDocument {
                id: "document-1".to_string(),
                scope: KnowledgeScope::Team,
                scope_ref: Some("team-1".to_string()),
                kind: KnowledgeDocumentKind::Runbook,
                title: "On-call runbook".to_string(),
                summary: "Restart steps".to_string(),
                body: "Restart the daemon if health stays red".to_string(),
                source_id: Some("source-1".to_string()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["ops".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        }

        fn upsert_knowledge_source(
            &self,
            input: KnowledgeSourceUpsertInput,
        ) -> Result<KnowledgeSource, FleetMcpError> {
            self.calls.borrow_mut().push("upsert_knowledge_source".to_string());
            Ok(KnowledgeSource {
                id: input.id.unwrap_or_else(|| "source-2".to_string()),
                kind: input.kind,
                label: input.label,
                uri: input.uri,
                scope: input.scope,
                scope_ref: input.scope_ref,
                sync_state: input.sync_state,
                last_synced_at: input.last_synced_at,
                metadata: input.metadata,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn create_knowledge_document(
            &self,
            input: KnowledgeDocumentCreateInput,
        ) -> Result<KnowledgeDocument, FleetMcpError> {
            self.calls.borrow_mut().push("create_knowledge_document".to_string());
            Ok(KnowledgeDocument {
                id: input.id.unwrap_or_else(|| "document-2".to_string()),
                scope: input.scope,
                scope_ref: input.scope_ref,
                kind: input.kind,
                title: input.title,
                summary: input.summary,
                body: input.body,
                source_id: input.source_id,
                source_kind: input.source_kind,
                tags: input.tags,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn list_knowledge_facts(
            &self,
            _input: KnowledgeRecordListInput,
        ) -> Result<Vec<KnowledgeFact>, FleetMcpError> {
            self.calls.borrow_mut().push("list_knowledge_facts".to_string());
            Ok(vec![KnowledgeFact {
                id: "fact-1".to_string(),
                scope: KnowledgeScope::Team,
                scope_ref: Some("team-1".to_string()),
                kind: KnowledgeFactKind::Policy,
                statement: "Marketing owns launch messaging".to_string(),
                confidence: 92,
                source_id: Some("source-1".to_string()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["policy".to_string()],
                observed_at: Utc::now(),
                created_at: Utc::now(),
            }])
        }

        fn create_knowledge_fact(
            &self,
            input: KnowledgeFactCreateInput,
        ) -> Result<KnowledgeFact, FleetMcpError> {
            self.calls.borrow_mut().push("create_knowledge_fact".to_string());
            Ok(KnowledgeFact {
                id: input.id.unwrap_or_else(|| "fact-2".to_string()),
                scope: input.scope,
                scope_ref: input.scope_ref,
                kind: input.kind,
                statement: input.statement,
                confidence: input.confidence,
                source_id: input.source_id,
                source_kind: input.source_kind,
                tags: input.tags,
                observed_at: input.observed_at.unwrap_or_else(Utc::now),
                created_at: Utc::now(),
            })
        }

        fn list_projects(&self, _input: ProjectListInput) -> Result<Vec<Project>, FleetMcpError> {
            self.calls.borrow_mut().push("list_projects".to_string());
            Ok(Vec::new())
        }

        fn create_project(&self, input: ProjectCreateInput) -> Result<Project, FleetMcpError> {
            self.calls.borrow_mut().push(format!("create_project:{}", input.slug));
            Ok(Project {
                id: "project-1".to_string(),
                team_id: input.team_id,
                slug: input.slug,
                root_path: input.root_path,
                ao_project_root: input.ao_project_root,
                default_branch: input.default_branch,
                enabled: input.enabled,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn list_schedules(
            &self,
            _input: ScheduleListInput,
        ) -> Result<Vec<Schedule>, FleetMcpError> {
            self.calls.borrow_mut().push("list_schedules".to_string());
            Ok(vec![Schedule {
                id: "schedule-1".to_string(),
                team_id: "team-1".to_string(),
                timezone: "UTC".to_string(),
                policy_kind: SchedulePolicyKind::AlwaysOn,
                windows: vec![WeekdayWindow {
                    weekdays: vec![0, 1, 2, 3, 4],
                    start_hour: 9,
                    end_hour: 17,
                }],
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        }

        fn create_schedule(&self, input: ScheduleCreateInput) -> Result<Schedule, FleetMcpError> {
            self.calls.borrow_mut().push(format!("create_schedule:{}", input.timezone));
            Ok(Schedule {
                id: "schedule-2".to_string(),
                team_id: input.team_id,
                timezone: input.timezone,
                policy_kind: input.policy_kind,
                windows: input.windows.into_iter().map(Into::into).collect(),
                enabled: input.enabled,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn reconcile_daemons(
            &self,
            input: DaemonReconcileInput,
        ) -> Result<DaemonReconcileResult, FleetMcpError> {
            self.calls.borrow_mut().push("reconcile_daemons".to_string());
            Ok(DaemonReconcileResult {
                evaluated_at: input.at.unwrap_or_else(Utc::now),
                applied: input.apply,
                decisions: vec![DaemonReconcileDecision {
                    team_id: "team-1".to_string(),
                    desired_state: DaemonDesiredState::Running,
                    backlog_count: 1,
                    schedule_ids: vec!["schedule-1".to_string()],
                }],
            })
        }
    }

    #[test]
    fn initialize_returns_server_info() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
            .expect("message handled")
            .expect("response returned");

        assert!(response.contains("\"ao-fleet\""));
        assert!(response.contains("\"tools\""));
    }

    #[test]
    fn tools_call_routes_to_handlers() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fleet.team.create","arguments":{"slug":"marketing","name":"Marketing","mission":"Own campaigns","ownership":"launchapp.dev","business_priority":10}}}"#,
            )
            .expect("message handled")
            .expect("response returned");

        let payload: serde_json::Value = serde_json::from_str(&response).expect("valid response");
        let text = payload["result"]["content"][0]["text"].as_str().expect("text content");

        assert!(text.contains("\"team-2\""), "response: {response}");
        assert!(text.contains("\"Marketing\""), "response: {response}");
    }

    #[test]
    fn tools_call_routes_fleet_overview() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fleet.overview","arguments":{"team_id":"team-1","at":"2025-03-03T10:00:00Z","backlog_by_team":{"team-1":3}}}}"#,
            )
            .expect("message handled")
            .expect("response returned");

        let payload: serde_json::Value = serde_json::from_str(&response).expect("valid response");
        let text = payload["result"]["content"][0]["text"].as_str().expect("text content");

        assert!(text.contains("\"team_count\": 1"), "response: {response}");
        assert!(text.contains("\"resume\""), "response: {response}");
    }

    #[test]
    fn tools_call_routes_knowledge_documents() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fleet.knowledge.document.list","arguments":{"scope":"team","scope_ref":"team-1","limit":10}}}"#,
            )
            .expect("message handled")
            .expect("response returned");

        let payload: serde_json::Value = serde_json::from_str(&response).expect("valid response");
        let text = payload["result"]["content"][0]["text"].as_str().expect("text content");

        assert!(text.contains("\"On-call runbook\""), "response: {response}");
        assert!(text.contains("\"scope_ref\": \"team-1\""), "response: {response}");
    }

    #[test]
    fn tools_call_routes_knowledge_source_upsert() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fleet.knowledge.source.upsert","arguments":{"kind":"manual_note","label":"Operator note","scope":"team","scope_ref":"team-1","sync_state":"ready","metadata":{"owner":"ops"}}}}"#,
            )
            .expect("message handled")
            .expect("response returned");

        let payload: serde_json::Value = serde_json::from_str(&response).expect("valid response");
        let text = payload["result"]["content"][0]["text"].as_str().expect("text content");

        assert!(text.contains("\"Operator note\""), "response: {response}");
        assert!(text.contains("\"sync_state\": \"ready\""), "response: {response}");
    }

    #[test]
    fn tools_list_returns_surface() {
        let server = FleetMcpServer::new(MockHandlers::default());
        let response = server
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#)
            .expect("message handled")
            .expect("response returned");

        assert!(response.contains("\"fleet.team.list\""));
        assert!(response.contains("\"fleet.daemon.reconcile\""));
        assert!(response.contains("\"fleet.overview\""));
        assert!(response.contains("\"fleet.knowledge.document.list\""));
        assert!(response.contains("\"fleet.knowledge.source.upsert\""));
    }
}
