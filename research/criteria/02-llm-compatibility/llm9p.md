# llm9p — LLM Compatibility

**Evidence source:** https://github.com/NERVsystems/llm9p README (Supported Backends table, backend differences table)
**Rating:** ~ partial

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so LLM compatibility here means "which LLMs can be served through the 9P interface." Currently only Anthropic models are supported (via direct API key or Claude Code CLI), with Ollama listed as planned. The pluggable backend architecture makes adding new providers tractable, but as of the README there is no OpenAI, Gemini, or open-weight backend available. From the client's perspective, any shell script or agent that can write to a 9P-mounted file can invoke the LLM without an SDK, which is genuinely language-agnostic; however, the LLM variety itself is narrow.
