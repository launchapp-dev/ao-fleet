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
                team_list_tool(),
                team_create_tool(),
                project_list_tool(),
                project_create_tool(),
                schedule_list_tool(),
                schedule_create_tool(),
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

fn daemon_reconcile_tool() -> McpToolDescriptor {
    McpToolDescriptor {
        name: "fleet.daemon.reconcile".to_string(),
        description: "Reconcile desired daemon state across the fleet".to_string(),
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
                "fleet.team.list",
                "fleet.team.create",
                "fleet.project.list",
                "fleet.project.create",
                "fleet.schedule.list",
                "fleet.schedule.create",
                "fleet.daemon.reconcile",
            ]
        );
    }

    #[test]
    fn tool_lookup_finds_tools() {
        let surface = FleetMcpSurface::new();

        assert!(surface.tool("fleet.team.create").is_some());
        assert!(surface.tool("fleet.missing").is_none());
    }
}
