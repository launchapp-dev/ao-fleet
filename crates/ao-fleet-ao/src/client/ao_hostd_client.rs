use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use reqwest::Method;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::models::hostd_host_profile::HostdHostProfile;
use crate::models::hostd_log_list_response::HostdLogListResponse;
use crate::models::hostd_project_summary::HostdProjectSummary;

#[derive(Debug, Clone)]
pub struct AoHostdClient {
    base_url: String,
    http: Client,
}

impl AoHostdClient {
    pub fn new(base_url: impl Into<String>, bearer_token: Option<String>) -> Result<Self> {
        let base_url = normalize_base_url(base_url.into())?;
        let http = Client::builder()
            .default_headers(build_headers(bearer_token)?)
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { base_url, http })
    }

    pub fn host_profile(&self) -> Result<HostdHostProfile> {
        let value = self.send(Method::GET, "/host")?;
        serde_json::from_value(value)
            .map_err(|error| anyhow!("failed to parse hostd host profile: {error}"))
    }

    pub fn list_projects(&self) -> Result<Vec<HostdProjectSummary>> {
        let value = self.send(Method::GET, "/projects")?;
        serde_json::from_value(value)
            .map_err(|error| anyhow!("failed to parse hostd project list: {error}"))
    }

    pub fn list_logs(
        &self,
        project_id: Option<&str>,
        after_seq: Option<u64>,
        limit: Option<usize>,
        cat: Option<&str>,
        level: Option<&str>,
        workflow: Option<&str>,
        run: Option<&str>,
    ) -> Result<HostdLogListResponse> {
        let mut query = Vec::new();
        if let Some(project_id) = project_id.filter(|value| !value.trim().is_empty()) {
            query.push(("project_id", project_id.to_string()));
        }
        if let Some(after_seq) = after_seq {
            query.push(("after_seq", after_seq.to_string()));
        }
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cat) = cat.filter(|value| !value.trim().is_empty()) {
            query.push(("cat", cat.to_string()));
        }
        if let Some(level) = level.filter(|value| !value.trim().is_empty()) {
            query.push(("level", level.to_string()));
        }
        if let Some(workflow) = workflow.filter(|value| !value.trim().is_empty()) {
            query.push(("workflow", workflow.to_string()));
        }
        if let Some(run) = run.filter(|value| !value.trim().is_empty()) {
            query.push(("run", run.to_string()));
        }

        let value = self.send_with_query(Method::GET, "/logs", &query)?;
        serde_json::from_value(value)
            .map_err(|error| anyhow!("failed to parse hostd log list: {error}"))
    }

    fn send(&self, method: Method, path: &str) -> Result<Value> {
        self.send_with_query(method, path, &[])
    }

    fn send_with_query(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.request(method, &url).query(query).send()?;
        let status = response.status();
        let body = response.text()?;
        let envelope: HostdEnvelope =
            serde_json::from_str(&body).map_err(|error| anyhow!("{url}: invalid JSON: {error}"))?;

        if !status.is_success() {
            bail!("{url}: hostd request failed with HTTP {status}: {}", envelope.error_message());
        }

        if !envelope.ok {
            bail!("{url}: hostd envelope reported ok=false: {}", envelope.error_message());
        }

        Ok(envelope.data.unwrap_or(Value::Null))
    }
}

#[derive(Debug, serde::Deserialize)]
struct HostdEnvelope {
    ok: bool,
    data: Option<Value>,
    error: Option<HostdErrorBody>,
}

impl HostdEnvelope {
    fn error_message(&self) -> String {
        self.error
            .as_ref()
            .map(|error| error.message.clone())
            .unwrap_or_else(|| "unknown hostd error".to_string())
    }
}

#[derive(Debug, serde::Deserialize)]
struct HostdErrorBody {
    message: String,
}

fn build_headers(bearer_token: Option<String>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    if let Some(token) = bearer_token.filter(|value| !value.trim().is_empty()) {
        let header_value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|error| anyhow!("invalid bearer token header: {error}"))?;
        headers.insert(AUTHORIZATION, header_value);
    }
    Ok(headers)
}

fn normalize_base_url(base_url: String) -> Result<String> {
    let trimmed = base_url.trim().trim_end_matches('/').to_string();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        bail!("hostd base URL must start with http:// or https://");
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
    fn parses_host_profile() {
        let server = test_server(
            StatusCode::OK,
            serde_json::json!({
                "ok": true,
                "data": {
                    "slug": "founder-mac",
                    "name": "Founder Mac",
                    "address": "http://founder.local:7444",
                    "platform": "macos",
                    "status": "healthy",
                    "capacity_slots": 6,
                    "fleet_url": "http://fleet.local:7600",
                    "project_count": 2
                }
            }),
        );
        let client = AoHostdClient::new(server, None).expect("client should build");

        let profile = client.host_profile().expect("profile should parse");

        assert_eq!(profile.slug, "founder-mac");
        assert_eq!(profile.project_count, 2);
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
