use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::fleet_error::FleetError;
use crate::models::daemon_desired_state::DaemonDesiredState;
use crate::models::daemon_override_mode::DaemonOverrideMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewDaemonOverride {
    pub team_id: String,
    pub mode: DaemonOverrideMode,
    pub forced_state: Option<DaemonDesiredState>,
    pub pause_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub source: String,
}

impl NewDaemonOverride {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.team_id.trim().is_empty() || self.source.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "daemon override team_id and source are required".to_string(),
            });
        }

        match self.mode {
            DaemonOverrideMode::ForceDesiredState => {
                if self.forced_state.is_none() {
                    return Err(FleetError::Validation {
                        message: "force desired state overrides require a forced_state".to_string(),
                    });
                }
                if self.pause_until.is_some() {
                    return Err(FleetError::Validation {
                        message: "force desired state overrides cannot set pause_until".to_string(),
                    });
                }
            }
            DaemonOverrideMode::FreezeUntil => {
                if self.pause_until.is_none() {
                    return Err(FleetError::Validation {
                        message: "freeze overrides require pause_until".to_string(),
                    });
                }
                if self.forced_state.is_some() {
                    return Err(FleetError::Validation {
                        message: "freeze overrides cannot set forced_state".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}
