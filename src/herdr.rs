use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone)]
pub enum HerdrEvent {
    AgentList { agents: Vec<SnapshotAgent> },
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
    // Verify socket is reachable
    UnixStream::connect(socket_path).ok()?;

    let (tx, rx) = mpsc::channel();
    let path = socket_path.to_string();

    thread::spawn(move || {
        loop {
            if let Some(agents) = poll_agent_list(&path) {
                if tx.send(HerdrEvent::AgentList { agents }).is_err() {
                    break;
                }
            }
            thread::sleep(Duration::from_secs(5));
        }
    });

    Some(rx)
}

fn poll_agent_list(socket_path: &str) -> Option<Vec<SnapshotAgent>> {
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;

    let req = serde_json::json!({
        "id": "poll",
        "method": "agent.list",
        "params": {}
    });
    writeln!(stream, "{}", req).ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;

    let parsed: Value = serde_json::from_str(line.trim()).ok()?;
    let result = parsed.get("result")?;
    let agents_val = result.get("agents")?.as_array()?;

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

    Some(agents)
}
