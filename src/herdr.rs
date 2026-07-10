use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum HerdrEvent {
    AgentDetected {
        pane_id: String,
        workspace_id: String,
        agent: Option<String>,
    },
    AgentStatusChanged {
        pane_id: String,
        workspace_id: String,
        agent_status: String,
        agent: Option<String>,
        custom_status: Option<String>,
    },
    PaneClosed {
        pane_id: String,
    },
    Snapshot {
        agents: Vec<SnapshotAgent>,
    },
}

#[derive(Debug, Clone)]
pub struct SnapshotAgent {
    pub pane_id: String,
    pub workspace_id: String,
    pub agent: Option<String>,
    pub agent_status: String,
    pub name: Option<String>,
    pub cwd: Option<String>,
}

pub fn connect(socket_path: &str) -> Option<mpsc::Receiver<HerdrEvent>> {
    let stream = UnixStream::connect(socket_path).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;

    let (tx, rx) = mpsc::channel();
    let mut write_stream = stream.try_clone().ok()?;

    let snapshot_req = serde_json::json!({
        "id": "snap_1",
        "method": "session.snapshot",
        "params": {}
    });
    writeln!(write_stream, "{}", snapshot_req).ok()?;

    let subscribe_req = serde_json::json!({
        "id": "sub_1",
        "method": "events.subscribe",
        "params": {
            "subscriptions": [
                { "type": "pane.agent_detected" },
                { "type": "pane.agent_status_changed" },
                { "type": "pane.closed" },
                { "type": "pane.exited" }
            ]
        }
    });
    writeln!(write_stream, "{}", subscribe_req).ok()?;

    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if line.is_empty() {
                continue;
            }
            let parsed: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(event) = parse_message(&parsed) {
                if tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    Some(rx)
}

fn parse_message(msg: &Value) -> Option<HerdrEvent> {
    if let Some(result) = msg.get("result") {
        if result.get("type").and_then(|t| t.as_str()) == Some("session_snapshot") {
            return parse_snapshot(result);
        }
    }

    if let Some(event_type) = msg.get("event").and_then(|e| e.as_str()) {
        let data = msg.get("data")?;
        return match event_type {
            "pane_agent_detected" | "pane.agent_detected" => parse_agent_detected(data),
            "pane_agent_status_changed" | "pane.agent_status_changed" => {
                parse_agent_status_changed(data)
            }
            "pane_closed" | "pane.closed" | "pane_exited" | "pane.exited" => {
                parse_pane_closed(data)
            }
            _ => None,
        };
    }

    None
}

fn parse_snapshot(result: &Value) -> Option<HerdrEvent> {
    let snapshot = result.get("snapshot")?;
    let agents_val = snapshot.get("agents")?.as_array()?;

    let agents: Vec<SnapshotAgent> = agents_val
        .iter()
        .filter_map(|a| {
            Some(SnapshotAgent {
                pane_id: a.get("pane_id")?.as_str()?.to_string(),
                workspace_id: a.get("workspace_id")?.as_str()?.to_string(),
                agent: a.get("agent").and_then(|v| v.as_str()).map(String::from),
                agent_status: a
                    .get("agent_status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                name: a.get("name").and_then(|v| v.as_str()).map(String::from),
                cwd: a.get("cwd").and_then(|v| v.as_str()).map(String::from),
            })
        })
        .collect();

    Some(HerdrEvent::Snapshot { agents })
}

fn parse_agent_detected(data: &Value) -> Option<HerdrEvent> {
    Some(HerdrEvent::AgentDetected {
        pane_id: data.get("pane_id")?.as_str()?.to_string(),
        workspace_id: data.get("workspace_id")?.as_str()?.to_string(),
        agent: data.get("agent").and_then(|v| v.as_str()).map(String::from),
    })
}

fn parse_agent_status_changed(data: &Value) -> Option<HerdrEvent> {
    Some(HerdrEvent::AgentStatusChanged {
        pane_id: data.get("pane_id")?.as_str()?.to_string(),
        workspace_id: data.get("workspace_id")?.as_str()?.to_string(),
        agent_status: data
            .get("agent_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        agent: data.get("agent").and_then(|v| v.as_str()).map(String::from),
        custom_status: data
            .get("custom_status")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

fn parse_pane_closed(data: &Value) -> Option<HerdrEvent> {
    Some(HerdrEvent::PaneClosed {
        pane_id: data.get("pane_id")?.as_str()?.to_string(),
    })
}
