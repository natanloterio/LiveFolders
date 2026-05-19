# llm9p — Observability

**Evidence source:** https://github.com/NERVsystems/llm9p README (`tokens` file, `-debug` flag, `context` file)
**Rating:** ✗ weak

llm9p exposes minimal observability: the `tokens` file reports the token count for the last response, and the `context` file shows the current conversation history as JSON. A `-debug` flag enables server-side logging to stderr. There is no persistent audit trail, no structured log of past requests and responses, no latency tracking beyond the implicit wall-clock time of file operations, and no metrics endpoint. For production or research use where interaction history matters, all observability must be built externally by the caller. The token count exposure is useful but narrow compared to full audit-trail systems.
