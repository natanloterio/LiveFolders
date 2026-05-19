# llm9p — Hot Reload

**Evidence source:** https://github.com/NERVsystems/llm9p README (Configuration, `model` file)
**Rating:** ~ partial

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so hot reload of tool definitions does not apply. However, within its own design, llm9p does support runtime reconfiguration without server restart: writing to the `model` file switches the active model, writing to `temperature` adjusts sampling, and writing to `system` replaces the system prompt — all taking effect on the next `ask` write. This is a lightweight form of hot reconfiguration at the LLM-parameter level. Adding or changing backend providers, however, requires restarting the server with different flags.
