use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::FleetError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewAuditEvent {
    pub team_id: Option<String>,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub summary: String,
    pub details: Value,
}

impl NewAuditEvent {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.entity_type.trim().is_empty()
            || self.entity_id.trim().is_empty()
            || self.action.trim().is_empty()
            || self.actor_type.trim().is_empty()
            || self.summary.trim().is_empty()
        {
            return Err(FleetError::Validation {
                message: "audit event fields cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}
