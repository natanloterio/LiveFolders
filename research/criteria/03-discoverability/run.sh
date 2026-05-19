#!/usr/bin/env bash
CRITERION="03-discoverability"
echo "[$CRITERION]"

echo ""
echo "--- LiveFoldersFS: auto-generated how_to.md (includes input type + constraints) ---"
cat << 'EOF'
# shout

Echoes input in uppercase.

## Files

- **shout** (`write_invoke`) — handler: `tr '[:lower:]' '[:upper:]'`, input: plain text, min_length: 1, max_length: 500
EOF

echo ""
echo "--- MCP: LLM receives list_tools JSON response ---"
cat << 'EOF'
{
  "tools": [
    {
      "name": "shout",
      "description": "Echoes input in uppercase.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "text": {"type": "string"}
        },
        "required": ["text"]
      }
    }
  ]
}
EOF

echo ""
echo "  LiveFoldersFS: human-readable markdown, LLM reads it naturally;"
echo "    input type, min/max length, pattern, and JSON schema now surfaced inline."
echo "  MCP: structured JSON schema, protocol-enforced parameter validation."
echo "  Winner: MCP retains schema-strictness edge; LiveFoldersFS now surfaces types + constraints."
