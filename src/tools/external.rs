use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::registry::{Session, Tool, ToolResult};

pub struct ExternalTool {
    name: String,
    dir: PathBuf,
    timeout_secs: u64,
}

impl ExternalTool {
    pub fn new(name: impl Into<String>, dir: PathBuf, timeout_secs: u64) -> Self {
        Self { name: name.into(), dir, timeout_secs }
    }

    fn endpoint_path(&self, endpoint: &str) -> PathBuf {
        self.dir.join(endpoint)
    }

    pub fn description_from_how_to(&self) -> String {
        let how_to = self.dir.join("how_to.md");
        std::fs::read_to_string(&how_to)
            .ok()
            .and_then(|s| s.lines().find(|l| !l.trim().is_empty()).map(|l| l.trim_start_matches('#').trim().to_string()))
            .unwrap_or_else(|| format!("External tool at {}", self.dir.display()))
    }

    pub fn endpoints_from_disk(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else { return vec![] };
        let mut eps = vec![];
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "how_to.md" { continue; }
            if path.is_dir() { continue; }
            let Ok(meta) = path.metadata() else { continue };
            if meta.permissions().mode() & 0o111 != 0 {
                eps.push(name);
            }
        }
        eps.sort();
        eps
    }
}

#[async_trait]
impl Tool for ExternalTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "External tool"
    }

    fn how_to(&self) -> &str {
        ""
    }

    fn endpoints(&self) -> Vec<&str> {
        vec![]
    }

    async fn invoke(&self, endpoint: &str, input: &[u8], _session: &Session) -> ToolResult {
        let script = self.endpoint_path(endpoint);

        let mut child = match Command::new(&script)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(&self.dir)
            .env("MODIXFS_TOOL", &self.name)
            .env("MODIXFS_ENDPOINT", endpoint)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return ToolResult::err(format!("failed to spawn {}: {}", script.display(), e)),
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input).await;
        }

        let result = tokio::time::timeout(
            Duration::from_secs(self.timeout_secs),
            child.wait_with_output(),
        )
        .await;

        match result {
            Err(_) => ToolResult::err("timeout"),
            Ok(Err(e)) => ToolResult::err(format!("process error: {}", e)),
            Ok(Ok(out)) => {
                if out.status.success() {
                    ToolResult::ok(out.stdout)
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    ToolResult::err(stderr.trim().to_string())
                }
            }
        }
    }
}
