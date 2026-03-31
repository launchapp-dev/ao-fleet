use anyhow::{Result, anyhow, bail};
use reqwest::Url;
use tungstenite::client::IntoClientRequest;
use tungstenite::http::HeaderValue;
use tungstenite::http::header::AUTHORIZATION;
use tungstenite::{Message, connect};

use crate::models::hostd_ws_event::HostdWsEvent;

#[derive(Debug, Clone)]
pub struct AoHostdWsClient {
    ws_url: String,
    bearer_token: Option<String>,
}

impl AoHostdWsClient {
    pub fn new(base_url: impl Into<String>, bearer_token: Option<String>) -> Result<Self> {
        Ok(Self { ws_url: normalize_ws_url(base_url.into())?, bearer_token })
    }

    pub fn stream_logs<F>(
        &self,
        project_id: Option<&str>,
        after_seq: Option<u64>,
        tail: Option<usize>,
        cat: Option<&str>,
        level: Option<&str>,
        workflow: Option<&str>,
        run: Option<&str>,
        mut on_event: F,
    ) -> Result<()>
    where
        F: FnMut(HostdWsEvent) -> Result<()>,
    {
        let mut url =
            Url::parse(&self.ws_url).map_err(|error| anyhow!("invalid hostd WS URL: {error}"))?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(project_id) = project_id.filter(|value| !value.trim().is_empty()) {
                query.append_pair("project_id", project_id);
            }
            if let Some(after_seq) = after_seq {
                query.append_pair("after_seq", &after_seq.to_string());
            }
            if let Some(tail) = tail {
                query.append_pair("tail", &tail.to_string());
            }
            if let Some(cat) = cat.filter(|value| !value.trim().is_empty()) {
                query.append_pair("cat", cat);
            }
            if let Some(level) = level.filter(|value| !value.trim().is_empty()) {
                query.append_pair("level", level);
            }
            if let Some(workflow) = workflow.filter(|value| !value.trim().is_empty()) {
                query.append_pair("workflow", workflow);
            }
            if let Some(run) = run.filter(|value| !value.trim().is_empty()) {
                query.append_pair("run", run);
            }
        }

        let mut request = url.as_str().into_client_request()?;
        if let Some(token) = self.bearer_token.as_deref().filter(|value| !value.trim().is_empty()) {
            let header_value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|error| anyhow!("invalid bearer token header: {error}"))?;
            request.headers_mut().insert(AUTHORIZATION, header_value);
        }

        let (mut socket, _) = connect(request)?;
        loop {
            match socket.read()? {
                Message::Text(payload) => {
                    let event: HostdWsEvent =
                        serde_json::from_str(payload.as_ref()).map_err(|error| {
                            anyhow!("failed to parse hostd websocket event: {error}")
                        })?;
                    on_event(event)?;
                }
                Message::Close(_) => return Ok(()),
                Message::Ping(payload) => socket.send(Message::Pong(payload))?,
                Message::Binary(_) | Message::Pong(_) | Message::Frame(_) => {}
            }
        }
    }
}

fn normalize_ws_url(base_url: String) -> Result<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if let Some(rest) = trimmed.strip_prefix("http://") {
        return Ok(format!("ws://{rest}/ws"));
    }
    if let Some(rest) = trimmed.strip_prefix("https://") {
        return Ok(format!("wss://{rest}/ws"));
    }

    bail!("hostd base URL must start with http:// or https://");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_http_base_url_to_ws() {
        let client =
            AoHostdWsClient::new("http://founder.local:7444", None).expect("client should build");

        assert_eq!(client.ws_url, "ws://founder.local:7444/ws");
    }
}
