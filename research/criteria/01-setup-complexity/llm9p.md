# llm9p — Setup Complexity

**Evidence source:** https://github.com/NERVsystems/llm9p README
**Rating:** ~ partial

Installing llm9p itself is a single `go install` command, and starting the server requires only an Anthropic API key and one flag (`./llm9p -addr :5640`). However, actually using the exposed filesystem requires a 9P client, and the README lists three options — plan9port, Infernode, or the Linux kernel `9p` module — each with its own installation path and non-trivial prerequisites (e.g., `sudo mount -t 9p` requires root, plan9port is a large suite, Infernode is a hosted Inferno OS environment). The protocol-level setup barrier is significantly higher than FUSE-based tools for users unfamiliar with Plan 9. The CLI backend path (via Claude Code CLI) adds another authentication dependency. Setup is easy for experienced Plan 9/Inferno users but steep for the typical AI-tooling developer.
