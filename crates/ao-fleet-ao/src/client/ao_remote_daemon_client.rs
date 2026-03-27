use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use reqwest::Method;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::models::daemon_command_result::DaemonCommandResult;
use crate::models::daemon_state::DaemonState;

const API_PREFIX: &str = "/api/v1";

#[derive(Debug, Clone)]
pub struct AoRemoteDaemonClient {
    base_url: String,
    http: Client,
}

impl AoRemoteDaemonClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let base_url = normalize_base_url(base_url.into())?;
        let http = Client::builder().timeout(Duration::from_secs(10)).build()?;
        Ok(Self { base_url, http })
    }

    pub fn daemon_status(&self) -> Result<DaemonState> {
        let value = self.send(Method::GET, "/daemon/status")?;
        let state = value
            .as_str()
            .ok_or_else(|| anyhow!("remote daemon status response was not a string"))?;
        Ok(DaemonState::from_cli_value(state))
    }

    pub fn start(&self) -> Result<DaemonCommandResult> {
        self.run_command("start", "/daemon/start")
    }

    pub fn stop(&self) -> Result<DaemonCommandResult> {
        self.run_command("stop", "/daemon/stop")
    }

    pub fn pause(&self) -> Result<DaemonCommandResult> {
        self.run_command("pause", "/daemon/pause")
    }

    pub fn resume(&self) -> Result<DaemonCommandResult> {
        self.run_command("resume", "/daemon/resume")
    }

    fn run_command(&self, command: &str, path: &str) -> Result<DaemonCommandResult> {
        Ok(DaemonCommandResult::from_cli_value(command, self.send(Method::POST, path)?))
    }

    fn send(&self, method: Method, path: &str) -> Result<Value> {
        let url = format!("{}{}{}", self.base_url, API_PREFIX, path);
        let response = self.http.request(method, &url).send()?;
        let status = response.status();
        let body = response.text()?;
        let envelope: AoRemoteEnvelope =
            serde_json::from_str(&body).map_err(|error| anyhow!("{url}: invalid JSON: {error}"))?;

        if !status.is_success() {
            bail!(
                "{url}: remote AO request failed with HTTP {status}: {}",
                envelope.error_message()
            );
        }

        if !envelope.ok {
            bail!("{url}: remote AO envelope reported ok=false: {}", envelope.error_message());
        }

        Ok(envelope.data.unwrap_or(Value::Null))
    }
}

#[derive(Debug, serde::Deserialize)]
struct AoRemoteEnvelope {
    ok: bool,
    data: Option<Value>,
    error: Option<AoRemoteErrorBody>,
}

impl AoRemoteEnvelope {
    fn error_message(&self) -> String {
        self.error
            .as_ref()
            .and_then(AoRemoteErrorBody::message)
            .unwrap_or_else(|| "unknown remote error".to_string())
    }
}

#[derive(Debug, serde::Deserialize)]
struct AoRemoteErrorBody {
    message: Option<String>,
    details: Option<Value>,
}

impl AoRemoteErrorBody {
    fn message(&self) -> Option<String> {
        self.message.clone().or_else(|| self.details.as_ref().map(Value::to_string))
    }
}

fn normalize_base_url(base_url: String) -> Result<String> {
    let trimmed = base_url.trim().trim_end_matches('/').to_string();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        bail!("remote AO base URL must start with http:// or https://");
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn rejects_non_http_remote_base_url() {
        assert!(AoRemoteDaemonClient::new("localhost:7777").is_err());
    }

    #[test]
    fn parses_remote_daemon_status() {
        let server = test_server(
            StatusCode::OK,
            serde_json::json!({
                "schema": "ao.cli.v1",
                "ok": true,
                "data": "paused"
            }),
        );
        let client = AoRemoteDaemonClient::new(server).expect("client should build");

        let status = client.daemon_status().expect("status should load");

        assert_eq!(status, DaemonState::Paused);
    }

    #[test]
    fn parses_remote_command_result() {
        let server = test_server(
            StatusCode::OK,
            serde_json::json!({
                "schema": "ao.cli.v1",
                "ok": true,
                "data": { "message": "daemon resumed" }
            }),
        );
        let client = AoRemoteDaemonClient::new(server).expect("client should build");

        let result = client.resume().expect("command should succeed");

        assert_eq!(result.command, "resume");
        assert_eq!(result.message.as_deref(), Some("daemon resumed"));
    }

    fn test_server(status: StatusCode, payload: Value) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener.local_addr().expect("local addr should exist");

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);

            let body = payload.to_string();
            let response = format!(
                "HTTP/1.1 {} {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("OK"),
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("response should write");
            stream.flush().expect("response should flush");
        });

        format!("http://{}", address)
    }
}
