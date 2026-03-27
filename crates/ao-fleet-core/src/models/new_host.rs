use crate::FleetError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewHost {
    pub slug: String,
    pub name: String,
    pub address: String,
    pub platform: String,
    pub status: String,
    pub capacity_slots: i32,
}

impl NewHost {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.slug.trim().is_empty()
            || self.name.trim().is_empty()
            || self.address.trim().is_empty()
        {
            return Err(FleetError::Validation {
                message: "host slug, name, and address are required".to_string(),
            });
        }

        if self.platform.trim().is_empty() || self.status.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "host platform and status are required".to_string(),
            });
        }

        if self.capacity_slots < 0 {
            return Err(FleetError::Validation {
                message: "host capacity must be non-negative".to_string(),
            });
        }

        Ok(())
    }
}
