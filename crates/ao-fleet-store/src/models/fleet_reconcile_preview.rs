use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::fleet_reconcile_preview_item::FleetReconcilePreviewItem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetReconcilePreview {
    pub evaluated_at: DateTime<Utc>,
    pub items: Vec<FleetReconcilePreviewItem>,
}
