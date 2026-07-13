# iris-agentic-dev Roadmap

Last updated: July 2026

---

## What's in master (unreleased)

These features are implemented and merged but not yet in a tagged release:

- **Server Manager discovery** — reads all IRIS servers from VS Code Server Manager
  keyring and settings; `IRIS_SERVER_NAME` pins a specific server
- **`iris_global`** — get/set/kill/list IRIS global nodes via MCP (PHI gate enforced)
- **`iris_doc` extensions** — fragment (partial source by line range), compiled (INT
  output), list (package document listing)
- **`iris_execute_method`** — invoke a ClassMethod directly without ObjectScript boilerplate
- **System observability** — five new `iris_admin` actions: active locks, running
  processes, journal search, namespace mappings, per-database status
- **Interoperability depth** — `iris_message_body`, `iris_business_rule_info`,
  `iris_production_diff`
- **SQL power extensions** — `iris_query` modes: `explain`, `count`, `write` (gated DML)
- **Durable telemetry** — queryable per-call telemetry replacing the in-memory ring buffer;
  native benchmark harness for contributors
- **CLI shortcuts** — `iris-agentic-dev exec`, `compile`, `query`, `doc`, `tool`
- **Windows native IRIS** — first-class support for IRIS installed natively on Windows
- **PHI policy and environment gates** — dispatch gate, mcpTemplate, dataPolicy,
  globalBlocklist, audit scrub

---

## Recently tagged (v0.8, July 2026)

- 30+ MCP tools: compile, execute, SQL, document, search, test, generate, interop, admin,
  SCM, debug
- VS Code extension — GitHub Copilot Agent mode integration (VS Code 1.99+)
- Skill regression harness — 100% lift on the repair benchmark with objectscript-review
- 87% test coverage, 3-version CI matrix (2023.1, 2025.1, 2026.1)

---

## Q3 2026 — Planned

### Official skill pack install

`iris-agentic-dev skill-install` writes curated SKILL.md files into your agent-skills
directory without needing a live IRIS connection. Covers the top benchmark-validated skills
for ObjectScript review, guardrails, and SQL patterns.

### ObjectScript line coverage

`iris_coverage_start` / `iris_coverage_stop` / `iris_coverage_report` wrap IRIS's
`%Monitor.System.LineByLine` to report which executable lines were hit during a test run.
Per-class and aggregate coverage, structured output.

---

## H2 2026 — Planned

### Editor integration E2E test suite

End-to-end tests verifying that iris-agentic-dev resolves the correct IRIS instance from
each editor's native MCP config — GitHub Copilot, Claude Code, OpenCode. Catches the class
of regression where missing config pins cause nondeterministic discovery fallthrough.

---

## 2027 — Exploring

### Skill optimizer loop

Close the loop between the benchmark harness, skill registry, and an optimization model:
run benchmark tasks, analyze execution traces, evolve skill text toward higher repair lift,
commit winning candidates automatically. Builds on the durable telemetry and coverage
substrates.

### Namespace and multi-instance workflows

First-class support for agents working across namespaces (USER / PROD / %SYS) or across
IRIS instances within a single session.

### Fine-tuned ObjectScript model integration

Surface a locally-runnable model fine-tuned on ObjectScript as an optional MCP tool —
`iris_generate_class` backed by a local model rather than a cloud LLM. Research tracked
separately in objectscript-coder.

---

## Contributing

See [BENCHMARKING.md](./light-skills/BENCHMARKING.md) to contribute a skill.
Issues and pull requests: [GitHub Issues](https://github.com/intersystems-community/iris-agentic-dev/issues)
Questions: [thomas.dyar@intersystems.com](mailto:thomas.dyar@intersystems.com)
