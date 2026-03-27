use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonStartOptions {
    pub autonomous: bool,
    pub skip_runner: bool,
    pub pool_size: Option<u32>,
    pub interval_secs: Option<u64>,
    pub auto_run_ready: Option<bool>,
}

impl Default for DaemonStartOptions {
    fn default() -> Self {
        Self {
            autonomous: true,
            skip_runner: false,
            pool_size: None,
            interval_secs: None,
            auto_run_ready: None,
        }
    }
}
