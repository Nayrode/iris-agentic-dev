# Feature Specification: CLI Tool Shortcuts

**Feature Branch**: `063-cli-tool-shortcuts`
**Created**: 2026-07-11
**Status**: Draft
**Input**: User description: "CLI shortcuts for all MCP tools — iris-agentic-dev exec, compile, query and equivalent subcommands that expose every MCP tool directly from the terminal without needing an MCP client. Motivated by the development friction of having no way to test iris_execute, iris_compile, iris_query etc. from the CLI. The shortcuts should accept the same parameters as the MCP tools, support --namespace, --container and other connection flags, and print output to stdout so they can be used in scripts. Should cover at minimum: exec (iris_execute), compile (iris_compile), query (iris_query), and doc (iris_doc). Nice to have: all other MCP tools accessible via a generic iris-agentic-dev tool <name> [args] fallback."

## Problem Statement

`iris-agentic-dev` is both a CLI binary and an MCP server. Every tool it exposes through MCP (`iris_execute`, `iris_compile`, `iris_query`, `iris_doc`, and dozens more) is accessible only through an MCP client — Claude Code, Cursor, OpenCode, or a raw stdio session. There is no way to invoke these tools from the terminal directly.

This creates friction in two specific situations: (1) a developer debugging the tools themselves has no quick way to test a single tool call without spinning up a full MCP client session, and (2) scripts and CI steps that need IRIS operations (compile a class, run a query, execute a snippet) cannot use `iris-agentic-dev` directly — they must arrange a full MCP session or reach for `docker exec iris session` instead.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Execute ObjectScript from the terminal (Priority: P1)

A developer debugging `iris_execute` (or writing a script that needs to run ObjectScript against IRIS) runs a single command from their terminal and sees the output immediately, with no MCP client involved.

**Why this priority**: This is the exact friction that motivated the feature. `exec` is the most general-purpose tool and the one most often needed in scripts and ad-hoc debugging.

**Independent Test**: Run `iris-agentic-dev exec 'write $ZVersion,!'` against a live IRIS instance and assert the IRIS version string appears on stdout with exit code 0.

**Acceptance Scenarios**:

1. **Given** a live IRIS instance is reachable, **When** `iris-agentic-dev exec 'write $ZVersion,!'` is run, **Then** stdout contains the IRIS version string and the exit code is 0.
2. **Given** a `--namespace` flag is passed, **When** the command runs, **Then** the code executes in the specified namespace rather than the default.
3. **Given** the ObjectScript code produces an error (e.g. `<UNDEFINED>`), **When** the command runs, **Then** the error text appears on stdout and the exit code is non-zero.
4. **Given** no IRIS instance is reachable, **When** the command runs, **Then** a clear connection error is printed to stderr and the exit code is non-zero.
5. **Given** output is piped to another command, **When** the command runs, **Then** stdout contains only the IRIS output with no decorative framing (no "Result:" prefix, no spinner), making it safe to pipe.
6. **Given** a multi-line ObjectScript script in a file, **When** `iris-agentic-dev exec --file script.os` is run, **Then** the file contents execute as if passed inline.

---

### User Story 2 - Compile a class file from the terminal (Priority: P1)

A developer running a CI script or a local build step compiles one or more `.cls` files into IRIS without writing a custom MCP client or resorting to `docker exec`.

**Why this priority**: Compilation is the most common IRIS automation step in CI. The existing `iris-agentic-dev compile` subcommand exists but reads from a config file — this story adds direct file argument support matching `iris_compile` behavior.

**Independent Test**: Run `iris-agentic-dev compile MyApp.MyClass.cls` against a live IRIS instance and assert the compile succeeds (exit 0) or reports errors (exit non-zero with error text on stdout).

**Acceptance Scenarios**:

1. **Given** a syntactically valid `.cls` file path, **When** `iris-agentic-dev compile <path>` is run, **Then** the class compiles successfully, stdout shows the compile result, and exit code is 0.
2. **Given** a `.cls` file with a syntax error, **When** the command runs, **Then** the error message and line number appear on stdout and exit code is non-zero.
3. **Given** multiple file paths are passed, **When** the command runs, **Then** all are compiled and results for each are reported.

---

### User Story 3 - Run a SQL query from the terminal (Priority: P2)

A developer or script needs to run a SQL query against IRIS and get results as plain text on stdout — for inspection, for piping to `jq`, or for CI assertions.

**Why this priority**: SQL queries are the second most common scripting need after code execution. P2 because `exec` can run SQL via ObjectScript too, but a dedicated `query` subcommand with tabular output is significantly more ergonomic for SQL-specific use.

**Independent Test**: Run `iris-agentic-dev query 'SELECT Name FROM %Dictionary.ClassDefinition WHERE Name %STARTSWITH "Config.Map" ORDER BY Name' --namespace %SYS` and assert the results appear on stdout as readable text.

**Acceptance Scenarios**:

1. **Given** a valid SQL query, **When** `iris-agentic-dev query '<sql>'` is run, **Then** results appear on stdout in a readable format and exit code is 0.
2. **Given** a query that returns no rows, **When** the command runs, **Then** stdout indicates zero results (not empty) and exit code is 0.
3. **Given** a SQL syntax error, **When** the command runs, **Then** the IRIS error message appears on stdout and exit code is non-zero.

---

### User Story 4 - Read/write class source via doc subcommand (Priority: P2)

A developer wants to read or write an IRIS class document directly from the terminal — to inspect a class source, to push a class body from a file, or to script document operations.

**Why this priority**: `iris_doc` is the tool most useful for scripting document-level operations. P2 because `exec` covers most scripting needs, but doc-level access is distinct enough to warrant its own surface.

**Acceptance Scenarios**:

1. **Given** a class name, **When** `iris-agentic-dev doc get <ClassName>` is run, **Then** the class source is printed to stdout.
2. **Given** a file path, **When** `iris-agentic-dev doc put <ClassName> --file <path>` is run, **Then** the file contents are written as the class document in IRIS.

---

### User Story 5 - Invoke any MCP tool by name via a generic fallback (Priority: P3)

A developer wants to invoke any `iris-agentic-dev` MCP tool by its exact name with JSON arguments — for tools without dedicated subcommands, for automation, or for one-off exploration.

**Why this priority**: There are 40+ MCP tools. Dedicated subcommands for all of them would be enormous scope. A generic fallback covers every tool immediately and degrades gracefully to the dedicated subcommand UX only where it matters most.

**Independent Test**: Run `iris-agentic-dev tool iris_info --args '{}'` and assert the IRIS instance info appears on stdout.

**Acceptance Scenarios**:

1. **Given** a valid MCP tool name and JSON args, **When** `iris-agentic-dev tool <name> --args '<json>'` is run, **Then** the tool's output appears on stdout and exit code reflects success/failure.
2. **Given** an unknown tool name, **When** the command runs, **Then** a list of valid tool names is printed to stderr and exit code is non-zero.

---

### Edge Cases

- What if the IRIS connection succeeds but the tool call itself errors (e.g. class not found, namespace doesn't exist)? Exit code must be non-zero; error text on stdout (matching what the MCP tool returns), not swallowed.
- What if `--container` is passed but Docker is not available on the host? Clear error on stderr distinguishing "Docker not found" from "container not found" from "IRIS not reachable in container."
- What if code passed to `exec` contains shell-special characters (quotes, semicolons, `$` signs)? The user is responsible for quoting; the tool must pass the argument through verbatim without additional escaping or interpretation.
- What if both `--namespace` and a namespace already baked into `--container`'s discovery differ? Explicit `--namespace` flag wins.
- What if output is very large (tens of thousands of lines)? stdout streaming — the output is not buffered in memory before printing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST expose at minimum four dedicated subcommands: `exec` (execute ObjectScript), `compile` (compile class files — file args override toml-based behavior), `query` (run SQL), and `doc` (get/put class documents).
- **FR-002**: Each dedicated subcommand MUST accept the same core connection flags: `--namespace`, `--container`, `--host`, `--port`, `--username`, `--password` — consistent with how the existing `compile` subcommand and the MCP tools resolve their IRIS connection today.
- **FR-003**: Each subcommand MUST print its primary output to stdout with no decorative framing, so output is safe to pipe to other commands.
- **FR-004**: Each subcommand MUST exit with code 0 on success and non-zero on any error (connection failure, compile error, runtime error, syntax error).
- **FR-005**: Error and diagnostic messages MUST go to stderr, not stdout, so they do not corrupt piped output.
- **FR-006**: The CLI MUST provide a generic `tool` subcommand that accepts any MCP tool name and a JSON argument string, invokes that tool, and prints its output to stdout — covering the full set of tools not addressed by dedicated subcommands.
- **FR-007**: The `tool` subcommand MUST print a list of valid tool names when given an unknown tool name, so discoverability does not require reading documentation.
- **FR-008**: All subcommands MUST use the same IRIS connection discovery cascade already implemented for the MCP server (env vars → named container → localhost scan → Docker scan), so no new connection configuration is required.
- **FR-009**: The `exec` subcommand MUST support reading code from two additional sources beyond an inline argument: `-` as the code argument reads from stdin (`cat script.os | iris-agentic-dev exec -`), and `--file <path>` reads from a file on disk (`iris-agentic-dev exec --file script.os`). Both are equivalent in behavior to passing the code inline.

### Key Entities

- **Subcommand**: A named CLI verb (`exec`, `compile`, `query`, `doc`, `tool`) directly under the `iris-agentic-dev` binary.
- **Connection Flags**: The set of flags (`--namespace`, `--container`, `--host`, etc.) that override IRIS connection discovery for a single invocation.
- **Tool Fallback**: The `tool <name> --args '<json>'` surface that routes to any MCP tool by its exact registered name.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can invoke `iris-agentic-dev exec 'write $ZVersion,!'` against a live IRIS instance and see the version string on stdout within 3 seconds, with no MCP client running.
- **SC-002**: A CI script can compile a `.cls` file and assert success or failure based solely on exit code, with no additional tooling.
- **SC-003**: Every MCP tool is reachable from the CLI via `iris-agentic-dev tool <name>` — no tool requires a running MCP client to invoke.
- **SC-004**: A developer can confirm which connection a subcommand is using (which IRIS instance, which namespace) within one additional command invocation (e.g. `iris-agentic-dev tool check_config --args '{}'`).
- **SC-005**: Output from all subcommands is pipe-safe: `iris-agentic-dev query '<sql>' | grep SomeClass` works correctly with no spurious lines on stdout.

## Assumptions

- The existing IRIS connection discovery logic is reused as-is — no new connection mechanism is introduced for CLI use.
- The dedicated subcommands (`exec`, `compile`, `query`, `doc`) are wrappers over the same underlying IRIS connection and tool logic the MCP server already uses, not separate reimplementations.
- The existing `compile` subcommand is extended in-place: with no file arguments it reads `iris-dev.toml` (existing behavior preserved); with one or more file arguments it compiles those files directly, ignoring `iris-dev.toml`. No new subcommand name is introduced.
- Output format for `query` is tab-separated columns with one row per line (standard Unix convention): single-column results print one value per line; multi-column results print tab-delimited fields per row with a tab-delimited header row first. No alignment padding, no box-drawing characters. Structured output (JSON, CSV) is out of scope for this spec but must not be architecturally precluded.

## Clarifications

### Session 2026-07-11

- Q: What format should `query` results use on stdout? → A: Tab-separated columns, one row per line — header row first, no alignment padding, no box-drawing.
- Q: How does `iris-agentic-dev compile <file>` relate to the existing `compile` subcommand (reads iris-dev.toml)? → A: Single `compile` subcommand — file args override toml behavior; no file args = existing toml behavior unchanged.
- Q: Should `exec` accept `--file <path>` in addition to `-` stdin? → A: Both — `--file <path>` and `-` stdin, equivalent in behavior to inline code argument.

## Out of Scope

- Interactive/REPL mode for any subcommand.
- Structured output formats (JSON, CSV) for query results — tab-separated plain text only (see Assumptions).
- Subcommands for every individual MCP tool (the generic `tool` fallback handles the long tail).
- Any change to the MCP server's tool behavior or protocol — these subcommands call the same underlying logic, not the MCP layer.
