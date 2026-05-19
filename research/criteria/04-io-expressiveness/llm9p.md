# llm9p — I/O Expressiveness

**Evidence source:** https://github.com/NERVsystems/llm9p README (Filesystem Schema, Streaming section)
**Rating:** ~ partial

The `ask` file implements a synchronous request-response pattern (write prompt, read response from the same file), and a `stream/` subdirectory provides chunk-by-chunk streaming via `stream/ask` and `stream/chunk`. Conversation context is maintained in `context` and system prompt in `system`, enabling multi-turn dialogue across file operations. However, inputs and outputs are plain text only — there is no support for binary payloads, structured JSON tool results, or multi-modal content. The interface is highly expressive for text-in/text-out LLM interaction but does not address richer I/O types needed for agentic tool use.
