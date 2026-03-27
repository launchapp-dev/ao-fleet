use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetReconcileAction {
    Keep,
    Start,
    StartPaused,
    Resume,
    Pause,
    Stop,
}
