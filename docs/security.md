# Security / Sandbox

Every tool handler runs in an isolated sandbox. Filesystem access is restricted to the paths the tool explicitly declares, and outbound network connections are blocked by default. This limits the blast radius if a handler misbehaves or is supplied malicious input.

## Platform details

| Platform | Mechanism |
|---|---|
| Linux (kernel ≥ 5.13) | Landlock LSM + seccomp socket filter |
| macOS | `sandbox-exec` (deprecated by Apple but functional on all current releases) |

Both platforms degrade gracefully: if the kernel or OS feature is unavailable, LiveFolders logs a warning and continues running without isolation.

## Network access

Network is denied by default. Add `network: true` to the `sandbox:` block in `folder.yaml` for any tool that needs to reach external services:

```yaml
name: weather
description: Get the weather forecast for any city.

sandbox:
  network: true

files:
  - name: forecast
    type: write_invoke
    handler: "curl -s \"https://wttr.in/$(cat -)?format=3\""
```

## Strict mode

By default, LiveFolders logs a warning and continues if sandboxing is unavailable. Set `mode: strict` in `livefolders.yaml` to refuse to mount instead:

```yaml
sandbox:
  mode: strict   # abort mount if Landlock/sandbox-exec cannot be applied
```
