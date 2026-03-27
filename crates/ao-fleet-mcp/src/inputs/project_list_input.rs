use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectListInput {
    pub team_id: Option<String>,
    pub enabled_only: bool,
}
