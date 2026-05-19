# llm9p — Composability

**Evidence source:** https://github.com/NERVsystems/llm9p README (Shell Scripting section, Why 9P section)
**Rating:** ~ partial

llm9p inverts the tool-integration direction: it exposes the LLM as a filesystem rather than exposing tools to the LLM, so this criterion does not apply directly. Within the Unix shell paradigm, llm9p composes naturally with standard tools: the README shows piping `echo "question" > /mnt/llm/ask` and reading with `cat`, enabling LLM calls inside shell scripts and pipelines. Multiple llm9p servers could in principle be mounted at different mount points and queried independently. However, there is no structured tool chaining, no mechanism to fan out to multiple models, and no graph of capabilities that can be orchestrated programmatically. Composability is present at the shell level but absent at the agent-framework level.
