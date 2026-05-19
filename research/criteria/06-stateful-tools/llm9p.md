# llm9p — Stateful Tools

**Evidence source:** https://github.com/NERVsystems/llm9p README (File Behaviors table, `context` and `new` files)
**Rating:** ~ partial

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so this criterion does not apply directly. Within its own scope, llm9p does maintain in-process conversation state across multiple `ask` writes, persisting history in the `context` virtual file and preserving the system prompt across resets. However, this state is entirely in-memory (no disk persistence beyond the process lifetime), and there is no mechanism for recording or auditing individual interaction turns. Stateful tool management in the agentic sense — tracking long-running subprocesses, retrying on failure, persisting intermediate results — is outside llm9p's scope.
