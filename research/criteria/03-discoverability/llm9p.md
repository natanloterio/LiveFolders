# llm9p — Discoverability

**Evidence source:** https://github.com/NERVsystems/llm9p README (Filesystem Schema section, `_example` file)
**Rating:** — N/A

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so this criterion does not apply directly. The filesystem schema (`ask`, `model`, `temperature`, `context`, `_example`, `stream/`) is fixed and hard-coded; there is no dynamic tool registry that an LLM or agent could enumerate. The `_example` file provides human-readable usage hints, which is a weak form of self-description for shell users, but this is not machine-readable capability discovery in the sense that MCP or LiveFoldersFS provide.
