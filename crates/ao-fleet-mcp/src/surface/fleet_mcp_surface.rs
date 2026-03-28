use ao_fleet_core::SchedulePolicyKind;
use serde::{Deserialize, Serialize};

use super::mcp_tool_descriptor::McpToolDescriptor;
use super::mcp_tool_input_schema::McpToolInputSchema;
use super::mcp_tool_property::McpToolProperty;
use super::mcp_tool_value_kind::McpToolValueKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetMcpSurface {
    pub namespace: String,
    pub version: String,
    pub supported_schedule_policy_kinds: Vec<SchedulePolicyKind>,
    pub tools: Vec<McpToolDescriptor>,
}

impl FleetMcpSurface {
    pub fn new() -> Self {
        Self {
            namespace: "fleet".to_string(),
            version: "v1".to_string(),
            supported_schedule_policy_kinds: vec![
                SchedulePolicyKind::AlwaysOn,
                SchedulePolicyKind::BusinessHours,
                SchedulePolicyKind::Nightly,
                SchedulePolicyKind::ManualOnly,
                SchedulePolicyKind::BurstOnBacklog,
            ],
            tools: vec![
                overview_tool(),
                daemon_status_tool(),
                knowledge_search_tool(),
                knowledge_source_list_tool(),
                knowledge_source_upsert_tool(),
                knowledge_document_list_tool(),
                knowledge_document_create_tool(),
                knowledge_fact_list_tool(),
                knowledge_fact_create_tool(),
                team_list_tool(),
                team_create_tool(),
                host_list_tool(),
                host_get_tool(),
                project_list_tool(),
                project_create_tool(),
                project_host_placement_list_tool(),
                project_host_placement_assign_tool(),
                project_host_placement_clear_tool(),
                schedule_list_tool(),
                schedule_create_tool(),
                daemon_override_list_tool(),
                daemon_override_set_tool(),
                daemon_override_clear_tool(),
                daemon_reconcile_tool(),
            ],
        }
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn tool(&self, name: &str) -> Option<&McpToolDescriptor> {
        self.tools.iter().find(|tool| tool.name == name)
    }
}

fn overview_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.overview".to_string(),
        description: "Summarize fleet inventory and reconcile preview data".to_string(),
        input_schema: schema_with_properties(
            "Summarize inventory plus desired-vs-observed reconcile preview".to_string(),
            vec![
                string_property("team_id", "Optional team filter", false, "team_marketing"),
                string_property(
                    "at",
                    "Optional RFC 3339 timestamp used for schedule evaluation",
                    false,
                    "2025-03-03T10:00:00Z",
                ),
                object_property(
                    "backlog_by_team",
                    "Optional backlog counts keyed by team id",
                    false,
                    serde_json::json!({ "team_marketing": 3 }),
                ),
                object_property(
                    "observed_state_by_team",
                    "Optional observed daemon states keyed by team id",
                    false,
                    serde_json::json!({ "team_marketing": "running" }),
                ),
            ],
            Vec::new(),
        ),
        tags: vec!["inventory".to_string(), "overview".to_string(), "reconcile".to_string()],
    }
}

fn daemon_status_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.status".to_string(),
        description: "List persisted daemon status for fleet projects".to_string(),
        input_schema: schema_with_properties(
            "Read last observed daemon status by project".to_string(),
            vec![string_property("team_id", "Optional team filter", false, "team_marketing")],
            Vec::new(),
        ),
        tags: vec!["daemon".to_string(), "read".to_string(), "status".to_string()],
    }
}

fn knowledge_source_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.source.list".to_string(),
        description: "List knowledge ingestion sources for the fleet".to_string(),
        input_schema: knowledge_record_list_schema(
            "List knowledge sources with optional scope filters".to_string(),
        ),
        tags: vec!["knowledge".to_string(), "read".to_string(), "source".to_string()],
    }
}

fn knowledge_search_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.search".to_string(),
        description: "Search knowledge documents and facts".to_string(),
        input_schema: schema_with_properties(
            "Search knowledge by scope, kind, tags, and text".to_string(),
            vec![
                enum_property(
                    "scope",
                    "Optional knowledge scope filter",
                    false,
                    vec!["global", "team", "project", "operational"],
                    "team",
                ),
                string_property(
                    "scope_ref",
                    "Optional team or project identifier",
                    false,
                    "team_marketing",
                ),
                array_property(
                    "document_kinds",
                    "Optional document kind filters",
                    false,
                    serde_json::json!(["runbook"]),
                ),
                array_property(
                    "fact_kinds",
                    "Optional fact kind filters",
                    false,
                    serde_json::json!(["policy"]),
                ),
                array_property(
                    "source_kinds",
                    "Optional source kind filters",
                    false,
                    serde_json::json!(["manual_note"]),
                ),
                array_property("tags", "Optional tag filters", false, serde_json::json!(["ops"])),
                string_property("text", "Optional case-insensitive text filter", false, "restart"),
                integer_property("limit", "Maximum records to evaluate per record type", false, 50),
            ],
            Vec::new(),
        ),
        tags: vec!["knowledge".to_string(), "read".to_string(), "search".to_string()],
    }
}

fn knowledge_document_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.document.list".to_string(),
        description: "List persisted knowledge documents for the fleet".to_string(),
        input_schema: knowledge_record_list_schema(
            "List knowledge documents with optional scope filters".to_string(),
        ),
        tags: vec!["knowledge".to_string(), "read".to_string(), "document".to_string()],
    }
}

fn knowledge_fact_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.fact.list".to_string(),
        description: "List persisted knowledge facts for the fleet".to_string(),
        input_schema: knowledge_record_list_schema(
            "List knowledge facts with optional scope filters".to_string(),
        ),
        tags: vec!["knowledge".to_string(), "read".to_string(), "fact".to_string()],
    }
}

fn knowledge_source_upsert_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.source.upsert".to_string(),
        description: "Create or update a knowledge ingestion source".to_string(),
        input_schema: schema_with_properties(
            "Create or update a knowledge source".to_string(),
            vec![
                string_property("id", "Optional source id for updates", false, "source_ops_note"),
                enum_property(
                    "kind",
                    "Knowledge source kind",
                    true,
                    vec![
                        "ao_event",
                        "git_commit",
                        "github_issue",
                        "github_pull_request",
                        "manual_note",
                        "incident",
                        "schedule_change",
                        "workflow_run",
                    ],
                    "manual_note",
                ),
                string_property("label", "Human-friendly source label", true, "Operator note"),
                string_property("uri", "Optional source URI", false, "file:///tmp/note.md"),
                enum_property(
                    "scope",
                    "Knowledge scope",
                    true,
                    vec!["global", "team", "project", "operational"],
                    "team",
                ),
                string_property(
                    "scope_ref",
                    "Optional team or project identifier",
                    false,
                    "team_marketing",
                ),
                enum_property(
                    "sync_state",
                    "Current ingestion sync state",
                    true,
                    vec!["pending", "ready", "stale", "failed"],
                    "ready",
                ),
                string_property(
                    "last_synced_at",
                    "Optional RFC 3339 timestamp for the last successful sync",
                    false,
                    "2025-03-03T10:00:00Z",
                ),
                object_property(
                    "metadata",
                    "Arbitrary source metadata",
                    true,
                    serde_json::json!({ "author": "ops" }),
                ),
            ],
            vec![
                "kind".to_string(),
                "label".to_string(),
                "scope".to_string(),
                "sync_state".to_string(),
                "metadata".to_string(),
            ],
        ),
        tags: vec!["knowledge".to_string(), "write".to_string(), "source".to_string()],
    }
}

fn knowledge_document_create_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.document.create".to_string(),
        description: "Create a persisted knowledge document".to_string(),
        input_schema: schema_with_properties(
            "Create a knowledge document".to_string(),
            vec![
                string_property("id", "Optional document id", false, "document_oncall_runbook"),
                enum_property(
                    "scope",
                    "Knowledge scope",
                    true,
                    vec!["global", "team", "project", "operational"],
                    "team",
                ),
                string_property(
                    "scope_ref",
                    "Optional team or project identifier",
                    false,
                    "team_marketing",
                ),
                enum_property(
                    "kind",
                    "Knowledge document kind",
                    true,
                    vec![
                        "brief",
                        "decision",
                        "runbook",
                        "research_note",
                        "team_profile",
                        "project_profile",
                        "incident_report",
                        "policy_note",
                    ],
                    "runbook",
                ),
                string_property("title", "Document title", true, "On-call runbook"),
                string_property("summary", "Short summary", true, "Restart steps"),
                string_property(
                    "body",
                    "Document body",
                    true,
                    "Restart the daemon if health stays red.",
                ),
                string_property("source_id", "Optional source id", false, "source_ops_note"),
                enum_property(
                    "source_kind",
                    "Optional source kind",
                    false,
                    vec![
                        "ao_event",
                        "git_commit",
                        "github_issue",
                        "github_pull_request",
                        "manual_note",
                        "incident",
                        "schedule_change",
                        "workflow_run",
                    ],
                    "manual_note",
                ),
                array_property("tags", "Tag list", true, serde_json::json!(["ops", "runbook"])),
            ],
            vec![
                "scope".to_string(),
                "kind".to_string(),
                "title".to_string(),
                "summary".to_string(),
                "body".to_string(),
                "tags".to_string(),
            ],
        ),
        tags: vec!["knowledge".to_string(), "write".to_string(), "document".to_string()],
    }
}

fn knowledge_fact_create_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.knowledge.fact.create".to_string(),
        description: "Create a persisted knowledge fact".to_string(),
        input_schema: schema_with_properties(
            "Create a knowledge fact".to_string(),
            vec![
                string_property("id", "Optional fact id", false, "fact_team_policy"),
                enum_property(
                    "scope",
                    "Knowledge scope",
                    true,
                    vec!["global", "team", "project", "operational"],
                    "team",
                ),
                string_property(
                    "scope_ref",
                    "Optional team or project identifier",
                    false,
                    "team_marketing",
                ),
                enum_property(
                    "kind",
                    "Knowledge fact kind",
                    true,
                    vec![
                        "policy",
                        "decision",
                        "risk",
                        "incident",
                        "workflow_outcome",
                        "schedule_observation",
                    ],
                    "policy",
                ),
                string_property(
                    "statement",
                    "Fact statement",
                    true,
                    "Marketing owns launch messaging",
                ),
                integer_property("confidence", "Confidence score from 0 to 100", true, 95),
                string_property("source_id", "Optional source id", false, "source_ops_note"),
                enum_property(
                    "source_kind",
                    "Optional source kind",
                    false,
                    vec![
                        "ao_event",
                        "git_commit",
                        "github_issue",
                        "github_pull_request",
                        "manual_note",
                        "incident",
                        "schedule_change",
                        "workflow_run",
                    ],
                    "manual_note",
                ),
                array_property("tags", "Tag list", true, serde_json::json!(["policy"])),
                string_property(
                    "observed_at",
                    "Optional RFC 3339 observation timestamp",
                    false,
                    "2025-03-03T10:00:00Z",
                ),
            ],
            vec![
                "scope".to_string(),
                "kind".to_string(),
                "statement".to_string(),
                "confidence".to_string(),
                "tags".to_string(),
            ],
        ),
        tags: vec!["knowledge".to_string(), "write".to_string(), "fact".to_string()],
    }
}

impl Default for FleetMcpSurface {
    fn default() -> Self {
        Self::new()
    }
}

fn team_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.team.list".to_string(),
        description: "List fleet teams and their ownership metadata".to_string(),
        input_schema: empty_schema("List teams".to_string()),
        tags: vec!["inventory".to_string(), "team".to_string()],
    }
}

fn knowledge_record_list_schema(description: String) -> McpToolInputSchema {
    schema_with_properties(
        description,
        vec![
            enum_property(
                "scope",
                "Optional knowledge scope filter",
                false,
                vec!["global", "team", "project", "operational"],
                "team",
            ),
            string_property(
                "scope_ref",
                "Optional team or project identifier for scoped knowledge",
                false,
                "team_marketing",
            ),
            integer_property("limit", "Maximum number of records to return", false, 100),
        ],
        Vec::new(),
    )
}

fn team_create_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.team.create".to_string(),
        description: "Create a team in the fleet registry".to_string(),
        input_schema: schema_with_properties(
            "Create a team with mission and ownership metadata".to_string(),
            vec![
                string_property("slug", "Team slug used as a stable identifier", true, "marketing"),
                string_property("name", "Human-friendly team name", true, "Marketing"),
                string_property(
                    "mission",
                    "What the team is responsible for delivering",
                    true,
                    "Own campaigns and launch messaging",
                ),
                string_property(
                    "ownership",
                    "What repos or capabilities the team owns",
                    true,
                    "launchapp.dev marketing site",
                ),
                integer_property(
                    "business_priority",
                    "Relative business priority for scheduling and resource allocation",
                    true,
                    50,
                ),
            ],
            vec![
                "slug".to_string(),
                "name".to_string(),
                "mission".to_string(),
                "ownership".to_string(),
                "business_priority".to_string(),
            ],
        ),
        tags: vec!["write".to_string(), "team".to_string()],
    }
}

fn host_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.host.list".to_string(),
        description: "List fleet hosts and their current metadata".to_string(),
        input_schema: empty_schema("List hosts".to_string()),
        tags: vec!["inventory".to_string(), "host".to_string()],
    }
}

fn host_get_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.host.get".to_string(),
        description: "Fetch a single fleet host by id".to_string(),
        input_schema: schema_with_properties(
            "Fetch a host by id".to_string(),
            vec![string_property("id", "Host id", true, "host_founder")],
            vec!["id".to_string()],
        ),
        tags: vec!["inventory".to_string(), "host".to_string(), "read".to_string()],
    }
}

fn project_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.project.list".to_string(),
        description: "List projects managed by the fleet".to_string(),
        input_schema: schema_with_properties(
            "List projects with optional team and status filters".to_string(),
            vec![
                string_property("team_id", "Filter projects by team id", false, "team_marketing"),
                boolean_property("enabled_only", "Only return enabled projects", false, true),
            ],
            Vec::new(),
        ),
        tags: vec!["inventory".to_string(), "project".to_string()],
    }
}

fn project_create_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.project.create".to_string(),
        description: "Register a project and bind it to a team".to_string(),
        input_schema: schema_with_properties(
            "Create a project binding for a team".to_string(),
            vec![
                string_property("team_id", "Owning team id", true, "team_marketing"),
                string_property("slug", "Stable project slug", true, "launchapp-www"),
                string_property(
                    "root_path",
                    "Local filesystem root for the repository",
                    true,
                    "/Users/me/projects/launchapp-www",
                ),
                string_property(
                    "ao_project_root",
                    "AO project root used by the daemon",
                    true,
                    "/Users/me/projects/launchapp-www",
                ),
                string_property("default_branch", "Default git branch", true, "main"),
                boolean_property(
                    "enabled",
                    "Whether the project is active in the fleet",
                    true,
                    true,
                ),
            ],
            vec![
                "team_id".to_string(),
                "slug".to_string(),
                "root_path".to_string(),
                "ao_project_root".to_string(),
                "default_branch".to_string(),
                "enabled".to_string(),
            ],
        ),
        tags: vec!["write".to_string(), "project".to_string()],
    }
}

fn project_host_placement_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.project.host.list".to_string(),
        description: "List project to host placements".to_string(),
        input_schema: schema_with_properties(
            "List project host placements with an optional team filter".to_string(),
            vec![string_property("team_id", "Optional team id filter", false, "team_marketing")],
            Vec::new(),
        ),
        tags: vec!["host".to_string(), "inventory".to_string(), "read".to_string()],
    }
}

fn project_host_placement_assign_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.project.host.assign".to_string(),
        description: "Assign a project to a host".to_string(),
        input_schema: schema_with_properties(
            "Assign a project to a host".to_string(),
            vec![
                string_property("project_id", "Project id", true, "project_launchapp"),
                string_property("host_id", "Host id", true, "host_founder"),
                string_property("assignment_source", "Source of the assignment", true, "founder"),
            ],
            vec!["project_id".to_string(), "host_id".to_string(), "assignment_source".to_string()],
        ),
        tags: vec!["host".to_string(), "inventory".to_string(), "write".to_string()],
    }
}

fn project_host_placement_clear_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.project.host.clear".to_string(),
        description: "Clear a project host placement".to_string(),
        input_schema: schema_with_properties(
            "Clear a project host placement".to_string(),
            vec![string_property("project_id", "Project id", true, "project_launchapp")],
            vec!["project_id".to_string()],
        ),
        tags: vec!["host".to_string(), "inventory".to_string(), "write".to_string()],
    }
}

fn schedule_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.schedule.list".to_string(),
        description: "List fleet schedules and activation windows".to_string(),
        input_schema: schema_with_properties(
            "List schedules with optional team filters".to_string(),
            vec![
                string_property("team_id", "Filter schedules by team id", false, "team_marketing"),
                boolean_property("enabled_only", "Only return enabled schedules", false, true),
            ],
            Vec::new(),
        ),
        tags: vec!["schedule".to_string(), "read".to_string()],
    }
}

fn schedule_create_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.schedule.create".to_string(),
        description: "Create a schedule policy for a team".to_string(),
        input_schema: schema_with_properties(
            "Create a schedule for a team".to_string(),
            vec![
                string_property("team_id", "Owning team id", true, "team_marketing"),
                string_property("timezone", "IANA timezone name", true, "America/Mexico_City"),
                enum_property(
                    "policy_kind",
                    "Schedule policy kind",
                    true,
                    vec![
                        "always_on",
                        "business_hours",
                        "nightly",
                        "manual_only",
                        "burst_on_backlog",
                    ],
                    "business_hours",
                ),
                array_property(
                    "windows",
                    "Schedule windows encoded as an array of weekday/hour objects",
                    true,
                    serde_json::json!([
                        { "weekdays": [0, 1, 2, 3, 4], "start_hour": 9, "end_hour": 17 }
                    ]),
                ),
                boolean_property("enabled", "Whether the schedule is active", true, true),
            ],
            vec![
                "team_id".to_string(),
                "timezone".to_string(),
                "policy_kind".to_string(),
                "windows".to_string(),
                "enabled".to_string(),
            ],
        ),
        tags: vec!["schedule".to_string(), "write".to_string()],
    }
}

fn daemon_override_list_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.override.list".to_string(),
        description: "List founder overrides for teams".to_string(),
        input_schema: schema_with_properties(
            "List overrides with an optional team filter".to_string(),
            vec![string_property("team_id", "Optional team id filter", false, "team_marketing")],
            Vec::new(),
        ),
        tags: vec!["daemon".to_string(), "override".to_string(), "read".to_string()],
    }
}

fn daemon_override_set_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.override.set".to_string(),
        description: "Create or update a founder override for a team".to_string(),
        input_schema: schema_with_properties(
            "Create or update a daemon override".to_string(),
            vec![
                string_property("team_id", "Owning team id", true, "team_marketing"),
                enum_property(
                    "mode",
                    "Override mode",
                    true,
                    vec!["force_desired_state", "freeze_until"],
                    "force_desired_state",
                ),
                enum_property(
                    "forced_state",
                    "Desired daemon state to force when the mode is force_desired_state",
                    false,
                    vec!["running", "paused", "stopped"],
                    "running",
                ),
                string_property(
                    "pause_until",
                    "RFC 3339 timestamp for freeze_until overrides",
                    false,
                    "2025-03-03T18:00:00Z",
                ),
                string_property(
                    "note",
                    "Human-readable founder note",
                    false,
                    "Pause for launch review",
                ),
                string_property("source", "Source label for the override", true, "founder"),
            ],
            vec!["team_id".to_string(), "mode".to_string(), "source".to_string()],
        ),
        tags: vec!["daemon".to_string(), "override".to_string(), "write".to_string()],
    }
}

fn daemon_override_clear_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.override.clear".to_string(),
        description: "Clear a founder override for a team".to_string(),
        input_schema: schema_with_properties(
            "Clear a daemon override".to_string(),
            vec![string_property("team_id", "Team id", true, "team_marketing")],
            vec!["team_id".to_string()],
        ),
        tags: vec!["daemon".to_string(), "override".to_string(), "write".to_string()],
    }
}

fn daemon_reconcile_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.reconcile".to_string(),
        description: "Reconcile desired daemon state across the fleet with reasons and overrides"
            .to_string(),
        input_schema: schema_with_properties(
            "Reconcile all daemon desired state against observed state".to_string(),
            vec![boolean_property(
                "dry_run",
                "Preview reconcile actions without executing them",
                false,
                false,
            )],
            Vec::new(),
        ),
        tags: vec!["daemon".to_string(), "reconcile".to_string()],
    }
}

fn empty_schema(description: String) -> McpToolInputSchema {
    McpToolInputSchema {
        description,
        properties: Vec::new(),
        required: Vec::new(),
        additional_properties: false,
    }
}

fn schema_with_properties(
    description: String,
    properties: Vec<McpToolProperty>,
    required: Vec<String>,
) -> McpToolInputSchema {
    McpToolInputSchema { description, properties, required, additional_properties: false }
}

fn string_property(
    name: &str,
    description: &str,
    required: bool,
    example: &str,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::String,
        description: description.to_string(),
        required,
        enum_values: Vec::new(),
        example: Some(serde_json::Value::String(example.to_string())),
    }
}

fn boolean_property(
    name: &str,
    description: &str,
    required: bool,
    example: bool,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::Boolean,
        description: description.to_string(),
        required,
        enum_values: Vec::new(),
        example: Some(serde_json::Value::Bool(example)),
    }
}

fn integer_property(
    name: &str,
    description: &str,
    required: bool,
    example: i64,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::Integer,
        description: description.to_string(),
        required,
        enum_values: Vec::new(),
        example: Some(serde_json::Value::Number(example.into())),
    }
}

fn enum_property(
    name: &str,
    description: &str,
    required: bool,
    enum_values: Vec<&str>,
    example: &str,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::Enum,
        description: description.to_string(),
        required,
        enum_values: enum_values.into_iter().map(ToString::to_string).collect(),
        example: Some(serde_json::Value::String(example.to_string())),
    }
}

fn array_property(
    name: &str,
    description: &str,
    required: bool,
    example: serde_json::Value,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::Array,
        description: description.to_string(),
        required,
        enum_values: Vec::new(),
        example: Some(example),
    }
}

fn object_property(
    name: &str,
    description: &str,
    required: bool,
    example: serde_json::Value,
) -> McpToolProperty {
    McpToolProperty {
        name: name.to_string(),
        kind: McpToolValueKind::Object,
        description: description.to_string(),
        required,
        enum_values: Vec::new(),
        example: Some(example),
    }
}

#[cfg(test)]
mod tests {
    use super::FleetMcpSurface;

    #[test]
    fn surface_has_expected_tool_names() {
        let surface = FleetMcpSurface::new();
        let names: Vec<_> = surface.tools.iter().map(|tool| tool.name.as_str()).collect();

        assert_eq!(
            names,
            vec![
                "fleet.overview",
                "fleet.daemon.status",
                "fleet.knowledge.search",
                "fleet.knowledge.source.list",
                "fleet.knowledge.source.upsert",
                "fleet.knowledge.document.list",
                "fleet.knowledge.document.create",
                "fleet.knowledge.fact.list",
                "fleet.knowledge.fact.create",
                "fleet.team.list",
                "fleet.team.create",
                "fleet.host.list",
                "fleet.host.get",
                "fleet.project.list",
                "fleet.project.create",
                "fleet.project.host.list",
                "fleet.project.host.assign",
                "fleet.project.host.clear",
                "fleet.schedule.list",
                "fleet.schedule.create",
                "fleet.daemon.override.list",
                "fleet.daemon.override.set",
                "fleet.daemon.override.clear",
                "fleet.daemon.reconcile",
            ]
        );
    }

    #[test]
    fn tool_lookup_finds_tools() {
        let surface = FleetMcpSurface::new();

        assert!(surface.tool("fleet.team.create").is_some());
        assert!(surface.tool("fleet.knowledge.document.list").is_some());
        assert!(surface.tool("fleet.missing").is_none());
    }
}
