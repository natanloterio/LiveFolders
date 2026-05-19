# llm9p — Security

**Evidence source:** https://github.com/NERVsystems/llm9p README (Usage, Configuration sections)
**Rating:** ✗ weak

llm9p binds a 9P server to a TCP port (default `:5640`) with no authentication by default; the README instructs users to mount with `-A` (anonymous authentication). Any process on the network that can reach that port can read and write the LLM interface, including resetting conversations or changing the model. The API key is passed only via environment variable to the server process, which is correct practice, but there is no access control, TLS support, or rate limiting described in the repository. For local development this is tolerable, but the design provides no security boundary suitable for multi-user or networked environments.
