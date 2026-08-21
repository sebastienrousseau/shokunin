# ssg-mcp

Model Context Protocol server on stdio, exposing ssg's RPC surface as MCP
tools.

MCP is line-delimited JSON-RPC 2.0. The surface an editor needs is small —
`initialize`, `tools/list`, `tools/call` — so this is hand-rolled rather
than taking an SDK dependency for three message shapes.

## Tools come from the registry

Nothing here declares tools. `#[ssg_rpc]` already registers every callable
with its input and output JSON Schema, and this walks that registry at
runtime, so a tool added to ssg appears over MCP with no second
declaration. A separate list would be a copy, and copies drift.

**A tool only appears if it is linked into the running binary.**
`inventory` collects at link time, so `tools/list` reflects what the host
binary registered — not what exists in the workspace.

## Usage

```bash
ssg-mcp < requests.jsonl
```

Configured as an MCP server, the client speaks the protocol directly.

## Errors

A tool that fails reports inside a successful result, with `isError: true`,
rather than as a JSON-RPC error. The model should be able to read the
failure and adapt; a protocol-level error surfaces to the client as a
broken server instead.
