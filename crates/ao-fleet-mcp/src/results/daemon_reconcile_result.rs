use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::results::daemon_reconcile_decision::DaemonReconcileDecision;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonReconcileResult {
    pub evaluated_at: DateTime<Utc>,
    pub applied: bool,
    pub decisions: Vec<DaemonReconcileDecision>,
}
