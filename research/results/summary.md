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

--- LiveFoldersFS: auto-generated how_to.md (includes input type + constraints) ---
# shout

Echoes input in uppercase.

## Files

- **shout** (`write_invoke`) — handler: `tr '[:lower:]' '[:upper:]'`, input: plain text, min_length: 1, max_length: 500

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

  LiveFoldersFS: human-readable markdown, LLM reads it naturally;
    input type, min/max length, pattern, and JSON schema now surfaced inline.
  MCP: structured JSON schema, protocol-enforced parameter validation.
  Winner: MCP retains schema-strictness edge; LiveFoldersFS now surfaces types + constraints.

=== criteria/04-io-expressiveness/ ===
[04-io-expressiveness]

Plain text:  both handle it
JSON:        LiveFoldersFS now enforces structural schema (required fields, property types)
             before handler runs; MCP enforces typed schema at protocol layer.
Multiline:   LiveFoldersFS native (stdin); MCP requires escaped string parameter
Binary:      LiveFoldersFS: pipe binary to handler; MCP: base64 encode in string (workaround)
Constraints: LiveFoldersFS: min/max length, regex pattern for strings; MCP: type/required only

  Example: json_reflect with schema {required:[text], properties:{text:{type:string}}}
    Valid input   {"text":"hello"} → passes, handler runs
    Missing field {}               → [ERROR:INVALID_INPUT] missing required field: 'text'
    Wrong type    {"text":42}      → [ERROR:INVALID_INPUT] field 'text' expected type 'string'

  Winner: MCP for protocol-enforced schema; LiveFoldersFS now competitive with
    opt-in structural validation (required fields, property types, string constraints).

=== criteria/05-security/ ===
[05-security]
  Both: run as user process, no OS sandboxing

  LiveFoldersFS (v0.7.0):
    - Opt-in per-endpoint structural validation via folder.yaml input.schema
    - Supports: required fields, property type checks, string min/max/pattern
    - Malformed input rejected before handler runs → no shell code executed
    - Remaining gap: opt-in (author must declare schema), not protocol-enforced
    - Shell injection within handler body is still the author's responsibility

  MCP:
    - Schema validation enforced unconditionally at protocol layer
    - Every tool rejects mistyped inputs automatically
    - No equivalent of string pattern or length constraints without custom validation

  Example (LiveFoldersFS search endpoint, schema: required=[query], query:string):
    Valid:   echo '{"query":"cats"}' → handler runs, returns result
    Missing: echo '{}'               → [ERROR:INVALID_INPUT] missing required field: 'query'
    Wrong:   echo '{"query":42}'     → [ERROR:INVALID_INPUT] field 'query' expected type 'string'

  Winner: MCP for unconditional protocol-layer enforcement;
    LiveFoldersFS ~ (partial, improved) — structural validation now available opt-in.

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

=== criteria/worked-example/ ===
[worked-example (users REST API)]
  LiveFoldersFS: 10 lines (folder.yaml)
  MCP (Python):  18 lines (server.py)

  LiveFoldersFS install: livefolders install github.com/natanloterio/LiveFolders/tree/master/examples/users
  MCP install: pip install mcp httpx && configure server in claude_desktop_config.json

  Winner: LiveFoldersFS — zero-dependency install vs multi-step MCP setup

