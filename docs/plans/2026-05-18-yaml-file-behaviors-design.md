# YAML File Behaviors Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:writing-plans to create the implementation plan from this design.

**Goal:** Allow tool authors to declare via `livefolders.yaml` how each virtual file responds to FUSE kernel calls (`open`, `read`, `write`), replacing the current implicit heuristics (executable bit detection, filename conventions).

**Architecture:** The `livefolders.yaml` manifest gains a `files` section that maps virtual file names to a `type` and an optional `handler` command. The FUSE layer reads this at mount time and routes `write`/`release`/`read` calls according to the declared type. Files with no manifest entry fall back to current behavior (backwards-compatible).

**Tech Stack:** Rust, `serde_yaml`, existing `Manifest` struct in `src/manifest.rs`, `LiveFolders` FUSE impl in `src/fs/vfs.rs`, `ExternalTool` in `src/tools/external.rs`.

---

## File Types

Four types cover all behaviors:

| Type | Write | Read |
|---|---|---|
| `write_invoke` | Invokes handler (blocking), stores result | Returns last result, clears it |
| `read_invoke` | Stores params (non-blocking) | Invokes handler with stored params (blocking), returns result |
| `passthrough` | Writes directly to disk | Reads directly from disk |
| `readonly` | Returns `EACCES` | Reads directly from disk |

`write_invoke` is the current default for executables. `passthrough` and `readonly` are the current defaults for regular and `how_to.md` files. `read_invoke` is new.

## State Machines

**write_invoke** (current behavior, now explicit):
```
IDLE → write(input) → release() → invoke(handler, stdin=input) [blocks] → READY → read() → IDLE
```

**read_invoke** (new):
```
IDLE ──────────────────────────────────────────→ read() → invoke(handler, stdin="") [blocks] → READY → IDLE
IDLE → write(params) → release() [no-op] → PARAMS_SET → read() → invoke(handler, stdin=params) [blocks] → READY → IDLE
```

## YAML Schema

The `files` section is added to `livefolders.yaml`:

```yaml
name: weather
description: Weather forecasts and search

files:
  - name: forecast
    type: read_invoke
    handler: ./bin/forecast          # stdin = params (or empty), stdout = result

  - name: search
    type: write_invoke
    handler: curl -s -X POST -d @- https://api.example.com/search

  - name: config.json
    type: passthrough                # no handler — direct disk I/O

  - name: how_to.md
    type: readonly                   # no handler — served from disk
```

**Rules:**
- `handler` is required for `write_invoke` and `read_invoke`.
- `handler` is forbidden for `passthrough` and `readonly`.
- `handler` is any shell command. LiveFolders passes input via stdin and reads output from stdout.
- Files not listed in `files` fall back to current heuristics (executable bit → `write_invoke`, regular file → `passthrough`, `how_to.md` → `readonly`).

## Handler Invocation

The handler receives:
- **stdin** — bytes written to the virtual file (empty if nothing was written for `read_invoke`)
- **env** — `LIVEFOLDERS_TOOL`, `LIVEFOLDERS_ENDPOINT`, plus all env vars present at mount time (including secrets)
- **cwd** — the tool directory

The handler returns:
- **stdout** — becomes the read result
- **stderr** — logged; returned as error content if exit code is non-zero

Timeout is inherited from the global `timeout` field in `tools.yaml` (default 30s).

## Handler Examples

```bash
# Local script
handler: ./bin/forecast

# Interpreter-based (no need for executable bit)
handler: python3 ./scripts/search.py

# HTTP via curl (stdin piped as POST body)
handler: curl -s -X POST -d @- https://api.example.com/search

# Reading from a config file inside the script is the handler's responsibility
handler: ./bin/query   # script may read ./config.json internally
```

## Backwards Compatibility

Tools without a `files` section in `livefolders.yaml` continue to work exactly as before:
- Executable files → `write_invoke`, handler = the file itself
- Regular files → `passthrough`
- `how_to.md` → `readonly`

This means all existing external tools require zero changes.

## Rust Changes

| File | Change |
|---|---|
| `src/manifest.rs` | Add `FileSpec` struct (`name`, `type`, `handler`) and `files: Vec<FileSpec>` to `Manifest` |
| `src/fs/vfs.rs` | In `release()` and `read()`: look up `FileSpec` from loaded manifest; dispatch based on `type` instead of executable bit |
| `src/tools/external.rs` | Expose `invoke_command(cmd, input, env, cwd, timeout)` helper usable for arbitrary handler strings |

No new files needed. The `FileType` enum lives in `src/manifest.rs`.
