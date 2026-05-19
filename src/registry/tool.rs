use async_trait::async_trait;

use super::Session;

pub struct ToolResult {
    pub output: Vec<u8>,
    pub error: Option<String>,
    /// Wall-clock time of the handler invocation in milliseconds.
    pub duration_ms: u64,
    /// Raw bytes written by the handler to stderr (empty on success or if not captured).
    pub stderr: Vec<u8>,
}

impl ToolResult {
    pub fn ok(output: impl Into<Vec<u8>>) -> Self {
        Self { output: output.into(), error: None, duration_ms: 0, stderr: Vec::new() }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { output: Vec::new(), error: Some(msg.into()), duration_ms: 0, stderr: Vec::new() }
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    /// Markdown content served at /tools/<name>/how_to.md
    fn how_to(&self) -> &str;

    /// Named endpoints under /tools/<name>/
    /// Each becomes a readable/writable virtual file.
    fn endpoints(&self) -> Vec<&str>;

    /// Called when the LLM writes to /tools/<name>/<endpoint>
    async fn invoke(&self, endpoint: &str, input: &[u8], session: &Session) -> ToolResult;
}
