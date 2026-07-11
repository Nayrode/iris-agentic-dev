# Research: CLI Tool Shortcuts (063)

## Existing CLI Structure

**Decision**: New subcommands follow the existing `cmd::compile` pattern — each gets a `cmd::<name>` module in `crates/iris-agentic-dev-bin/src/cmd/`, added as a variant in the `Commands` enum in `main.rs`.

**Rationale**: The `Compile` subcommand is already a working thin CLI wrapper over IRIS connection + tool logic. It reads `.iris-dev.toml`, builds an `IrisConnection`, calls the core lib, and prints results. Reusing this exact pattern avoids reinventing connection bootstrapping.

**Existing `Commands` variants**: `Mcp`, `Compile`, `Init`, `Install`, `Benchmark`, `External`.

---

## Connection Bootstrapping

**Decision**: New CLI subcommands bootstrap their IRIS connection identically to `cmd::compile`:
1. Read `.iris-dev.toml` (optional — falls back to discovery if absent).
2. Override with explicit CLI flags (`--namespace`, `--container`, `--host`, `--port`, `--username`, `--password`).
3. Call `IrisConnection::new(base_url, namespace, username, password, DiscoverySource)`.

**Rationale**: The same discovery cascade already works for `compile`. No new connection code needed.

---

## Tool Handler Access

**Decision**: CLI subcommands call the same underlying core library functions the MCP tools use — not through the MCP layer. The `IrisTools` struct methods annotated with `#[tool]` wrap the actual logic; CLI callers instantiate `IrisTools` (or call the inner logic functions directly if exposed) the same way the MCP server does.

**Rationale**: There is no existing generic "call tool by name" dispatch. Rather than building one for the short list of dedicated subcommands, each shortcut module calls the relevant core function directly. The generic `tool` fallback (FR-006) is the one place that needs a dispatch map.

**Verified against live IRIS** (iris-dev-iris, 2026.2): The new plain-ClassMethod `execute_via_generator` path (merged in fix/iris-execute-macro-preprocessor) correctly compiles and runs user ObjectScript with full macro preprocessor context. The `iris_compile`, `iris_query`, and `iris_doc` code paths all use pure HTTP Atelier REST — no Docker dependency.

---

## Generic `tool` Fallback Dispatch

**Decision**: The `tool <name> --args '<json>'` subcommand builds a dispatch map at startup: a `HashMap<&str, fn(IrisConnection, serde_json::Value) -> anyhow::Result<String>>` keyed by tool name. Each entry is a thin lambda calling the same core function the MCP handler calls.

**Rationale**: There is no existing dispatch path. A static HashMap is the simplest approach — no runtime reflection, no MCP overhead, exhaustive at compile time. If a tool name isn't in the map, the command lists all keys and exits non-zero (FR-007).

**Alternative considered**: Spin up an embedded MCP stdio session and send a `tools/call` JSON-RPC message. Rejected — adds latency, complexity, and requires the full server lifecycle for a CLI one-shot.

---

## Output Format

**`exec`**: Raw IRIS stdout output, verbatim. No header, no framing. Trailing newline from IRIS sentinel preserved.

**`query`**: Tab-separated columns, one row per line. First line is the header (column names, tab-separated). No alignment padding, no box-drawing. Confirmed: Atelier `/action/query` returns `result.content` as an array of objects with string keys → trivially serializable to TSV. Verified column names available from `result.columns` in the Atelier response.

**`compile`**: One line per compiled class: `OK: ClassName` or `ERROR: ClassName: <message>`. Exit 0 only if all classes compile without errors.

**`doc get`**: Raw class source text (the `content` array from Atelier GET joined as lines). Suitable for piping to a file or diff.

**`tool`**: Raw JSON tool result text, same as what the MCP `CallToolResult` content[0].text contains.

---

## Constitution Check Pre-Research

- **I. Zero-Install**: All new subcommands are pure Rust in the existing binary. No new install step. PASS.
- **II. ObjectScript Sanity**: No new ObjectScript APIs introduced. `exec`, `compile`, `query`, `doc` all use existing verified HTTP paths. PASS.
- **III. HTTP-First**: All shortcuts use Atelier REST (HTTP). Docker exec remains the fallback only. PASS.
- **IV. Test-First**: Unit tests for argument parsing and output formatting (no IRIS needed); live integration tests for each subcommand. PASS (plan includes test tasks first).
- **V. Output Shape Parity**: Documented above per subcommand. PASS.
- **VI. Environment Guard**: `exec` can run arbitrary ObjectScript — same write-gate logic that applies to `iris_execute` MCP tool applies here. PASS (reuse existing `is_write_allowed()` gate).
- **VII. Dependency Minimalism**: No new crates needed. Clap derive (already used), serde_json (already used), anyhow (already used). PASS.
- **VIII. 90% Coverage Gate**: Polish phase includes coverage check task. PASS.
