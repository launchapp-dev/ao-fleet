use crate::FleetError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTeam {
    pub slug: String,
    pub name: String,
    pub mission: String,
    pub ownership: String,
    pub business_priority: i32,
}

impl NewTeam {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.slug.trim().is_empty() || self.name.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "team slug and name are required".to_string(),
            });
        }

        if self.business_priority < 0 {
            return Err(FleetError::Validation {
                message: "business priority must be non-negative".to_string(),
            });
        }

        Ok(())
    }
}
