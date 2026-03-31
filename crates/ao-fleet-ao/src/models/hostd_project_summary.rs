use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostdProjectSummary {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub ao_project_root: String,
    pub default_branch: String,
    pub remote_url: Option<String>,
    pub has_git: bool,
    pub has_ao: bool,
    pub enabled: bool,
    pub source: String,
    pub ao_bin: Option<String>,
}
