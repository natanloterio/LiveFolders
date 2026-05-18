# External Tools & Hot-Reload Design

**Date:** 2026-05-18
**Status:** Approved

## Goal

Allow developers in any language to build ModixFS tools without writing Rust or recompiling. Tools are directories of scripts and files on disk. ModixFS discovers and hot-reloads them at runtime.

---

## Section 1 — File behavior model

Every file in a tool directory gets a behavior based on its name, executable bit, and type:

| File | Behavior |
|---|---|
| `how_to.md` | Read-only. Serves file content as documentation. |
| Executable (`chmod +x`) | Write → piped to stdin of the script. Stdout captured as result. Read → returns result, then clears. Stderr + non-zero exit → surfaced as error. |
| Regular file (not executable) | Passthrough to disk. Read → serves file content. Write → writes to disk. Permissions on disk are respected (`chmod 444` = read-only, etc). |
| Subdirectory | Becomes a virtual subdirectory in the mount. |

The LLM can also **create new files and directories** inside the mount:

| Operation | Behavior |
|---|---|
| `create` | Creates file on disk in the tool directory |
| `mkdir` | Creates directory on disk — new tool if directly under `/tools/` |
| `unlink` | Deletes file on disk |
| `rename` | Renames file on disk |
| `chmod` (setattr) | Changes permissions on disk — making a file executable promotes it to a subprocess endpoint immediately |

This enables the LLM to compose new tools at runtime:

```bash
mkdir /tools/mynewtool
echo "# My New Tool..." > /tools/mynewtool/how_to.md
cat > /tools/mynewtool/fetch << 'EOF'
#!/bin/bash
curl -s "https://api.example.com" -d "$(cat -)"
EOF
chmod +x /tools/mynewtool/fetch
# tool is immediately live
```

**Example tool directory:**

```
~/.config/modixfs/tools/
└── dataprep/
    ├── how_to.md          ← docs
    ├── fetch              ← executable: fetches data, writes output.csv, returns summary
    ├── output.csv         ← passthrough read: LLM inspects results
    ├── run.log            ← passthrough read+write: progress log
    └── config.json        ← passthrough read+write: LLM updates fetch parameters
```

---

## Section 2 — Directory structure & `tools.yaml`

New field `tools_dir` in `tools.yaml`:

```yaml
mount: /tmp/modixfs
tools_dir: ~/.config/modixfs/tools   # external tools discovered here
timeout: 30                          # global subprocess timeout in seconds

tools:                               # built-in Rust tools — unchanged
  - name: echo
  - name: github
    token_env: GITHUB_TOKEN
```

The `tools_dir` on disk mirrors `/tools/` in the mount directly. Built-in Rust tools and external script tools coexist. If both define the same name, built-in takes precedence.

---

## Section 3 — Hot-reload mechanism

ModixFS watches `tools_dir` using the `notify` crate (`inotify` on Linux, `kqueue` on macOS). The watcher runs as a background Tokio task.

**Events and actions:**

| Event | Action |
|---|---|
| Directory created under `tools_dir` | Register new external tool |
| Directory deleted | Unregister tool |
| File created in tool dir | Add endpoint or passthrough file |
| File deleted | Remove endpoint |
| `chmod` on file | Re-evaluate executable bit — promote/demote between endpoint and passthrough |
| `how_to.md` modified | Update docs served on next read |

**Registry change:**
`Arc<ToolRegistry>` → `Arc<RwLock<ToolRegistry>>`

FUSE handlers acquire a read lock per request. The watcher acquires a write lock only during registration changes.

**Edge case:** if the LLM is mid-write to an endpoint while the file is modified on disk, the in-progress write completes against the old version. The new version is used on the next invocation. Acceptable eventual consistency.

---

## Section 4 — Subprocess execution model

**Invocation:**
- Input bytes → `stdin`
- `stdout` → captured as result
- `stderr` + non-zero exit → `ERROR: <stderr>` returned as result
- Working directory → the tool directory on disk (relative paths like `./output.csv` work)
- Timeout → 30s default, kills process, result is `ERROR: timeout`

**Environment:**
- Inherits ModixFS's environment (tokens set at launch are available to scripts)
- Two extra vars injected:
  - `MODIXFS_TOOL` — tool name
  - `MODIXFS_ENDPOINT` — endpoint name

Single script can handle multiple endpoints:

```bash
#!/bin/bash
case "$MODIXFS_ENDPOINT" in
  create_issue) ... ;;
  search)       ... ;;
esac
```

**Security note:** scripts run with the same privileges as the `modixfs` process. The LLM can create and `chmod +x` new scripts which execute on next invocation. This is intentional (compose power) but users should be aware.

---

## Out of scope (for now)

- Per-tool timeout override in `tools.yaml`
- Multiple `tools_dir` search paths
- Sandboxing subprocess execution
- HTTP passthrough endpoint files (`.url` convention) — revisit later
