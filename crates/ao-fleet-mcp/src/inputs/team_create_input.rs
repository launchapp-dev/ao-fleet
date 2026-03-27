use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamCreateInput {
    pub slug: String,
    pub name: String,
    pub mission: String,
    pub ownership: String,
    pub business_priority: i32,
}
