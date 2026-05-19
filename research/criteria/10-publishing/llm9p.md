# llm9p — Publishing

**Evidence source:** https://github.com/NERVsystems/llm9p README (Installation section, Related Projects)
**Rating:** — N/A

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so this criterion does not apply directly. The project itself is published as a Go module (`go install github.com/NERVsystems/llm9p/cmd/llm9p@latest`), which is adequate for distributing the server binary. There is no concept of publishing or sharing tool definitions, capability manifests, or skill packages, because llm9p does not define any such artifacts. The related Infernode project can mount llm9p as a 9P service, which provides a form of service composition, but not a structured publishing ecosystem for tools.
