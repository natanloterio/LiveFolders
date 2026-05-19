# Experiment Results — 2026-05-19

=== criteria/01-setup-complexity/ ===
[01-setup-complexity]
  LiveFoldersFS: 6 lines (folder.yaml only, no Python)
  MCP (Python):  8 lines (server.py)
  Winner: LiveFoldersFS

=== criteria/02-llm-compatibility/ ===
[02-llm-compatibility]
  LiveFoldersFS: works on any host with bash/shell tool access
  MCP:           works on any host that implements MCP client protocol
  Winner: MCP for cross-host portability (MCP-native hosts); LiveFoldersFS for shell-capable agents.

=== criteria/03-discoverability/ ===
[03-discoverability]

--- LiveFoldersFS: LLM reads index.md (plain text) ---
# Tools

## shout
Echoes input in uppercase.

Files: shout

--- MCP: LLM receives list_tools JSON response ---
{
  "tools": [
    {
      "name": "shout",
      "description": "Echoes input in uppercase.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "text": {"type": "string"}
        },
        "required": ["text"]
      }
    }
  ]
}

  LiveFoldersFS: human-readable markdown, LLM reads it naturally
  MCP: structured JSON schema, enables parameter validation
  Winner: Tie — MCP wins on schema strictness; LiveFoldersFS wins on readability

=== criteria/04-io-expressiveness/ ===
[04-io-expressiveness]

Plain text:  both handle it
JSON:        LiveFoldersFS passes raw string (handler must parse); MCP enforces typed schema
Multiline:   LiveFoldersFS native (stdin); MCP requires escaped string parameter
Binary:      LiveFoldersFS: pipe binary to handler; MCP: base64 encode in string (workaround)

  Winner: MCP for structured/typed I/O; LiveFoldersFS for raw/binary/streaming

=== criteria/05-security/ ===
[05-security]
  Both: run as user process, no OS sandboxing
  LiveFoldersFS: shell handler = injection risk if input not sanitized in handler
  MCP: schema validation provides input sanitization layer
  Winner: MCP (marginal) — schema reduces injection surface

=== criteria/06-stateful-tools/ ===
[06-stateful-tools]

LiveFoldersFS: state lives in a file (e.g. /tmp/lf_counter) — persists across restarts
  Limitation: no atomic update without a lock file; concurrent writes can corrupt state

MCP: state lives in Python process memory — fast, no locking needed for single-threaded
  Limitation: state lost on server restart; persistent state still requires a file/DB

  Winner: MCP for in-process state; LiveFoldersFS for file-persisted state

=== criteria/07-composability/ ===
[07-composability]

LiveFoldersFS: compose via shell pipes — any unix tool is composable
  Cross-tool calls require writing to another endpoint file (awkward for chaining)

MCP: compose via Python function calls — clean, typed, testable
  Cross-server tool calls not natively supported; requires LLM orchestration

  Winner: MCP for within-server composition; LiveFoldersFS for unix pipeline composition

=== criteria/08-observability/ ===
[08-observability]
  LiveFoldersFS: stderr → log file; errors returned as plain text to LLM
  MCP: structured error objects; Python exceptions auto-converted
  Winner: MCP (marginal) — structured errors easier to handle programmatically

=== criteria/09-hot-reload/ ===
[09-hot-reload]

LiveFoldersFS:
  Edit folder.yaml or handler script → inotify watcher detects change → immediate
  No restart required. New file reads reflect updated handler within ~1s.

MCP:
  Edit server.py → must restart MCP server process → Claude Code must reconnect
  Restart latency: ~1-3s for Python startup + reconnect handshake

  Winner: LiveFoldersFS — true hot-reload via filesystem watcher

=== criteria/10-publishing/ ===
[10-publishing]

LiveFoldersFS:
  1. Add folder.yaml to any GitHub repo
  2. Done. Users install with: livefolders install github.com/you/repo
  No registry. No npm publish. No PyPI upload.

MCP:
  Option A: publish to npm/PyPI, users add to claude_desktop_config.json manually
  Option B: list in community MCP registry (no official registry yet)
  Option C: share repo URL, users clone and configure themselves

  Winner: LiveFoldersFS — one-command install from any GitHub URL

