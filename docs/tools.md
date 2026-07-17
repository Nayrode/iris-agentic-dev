# Tools

Most tools work over the Atelier REST API and connect to any IRIS instance — no Docker
required unless noted. Tools marked ✦ require `IRIS_CONTAINER`. Tools marked 🔒 are
write-gated (suppressed on Live instances unless `IRIS_ALLOW_PROD=1`).

---

## Code

| Tool                    | What it does                                                                                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `iris_compile`          | Compile a class, routine, or wildcard. Returns errors with line numbers.                                            |
| `iris_doc`              | Read, write, delete, insert, or check any IRIS document. Supports stale-edit guards via `expected`.                 |
| `iris_execute`          | Run ObjectScript, return output.                                                                                    |
| `iris_execute_method`   | Invoke a `ClassMethod` directly by class+method+args, no boilerplate. String-returning methods only (v1).           |
| `iris_query`            | Execute SQL, return rows as JSON. `mode=explain\|count\|write` for query plans, row-count estimates, and gated DML. |
| `iris_test`             | Run `%UnitTest` tests, return structured pass/fail results.                                                         |
| `iris_global`           | Read, write, kill, or list IRIS global nodes. PHI and system-blocklist gates enforced.                              |
| `iris_source_control` ✦ | Check lock status, checkout, execute SCM actions. CheckIn is opt-in via `IRIS_SCM_ALLOW_CHECKIN=1`.                 |

---

## Search and introspection

| Tool                            | What it does                                                                                                                                                   |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `iris_symbols`                  | Search classes and methods via `%Dictionary`.                                                                                                                  |
| `iris_symbols_local`            | Search `.cls`/`.mac`/`.inc` files on disk by glob pattern — no IRIS connection required.                                                                       |
| `docs_introspect`               | Deep class inspection: methods, properties, XData, superclasses.                                                                                               |
| `iris_search`                   | Full-text search across the namespace. Supports regex, category filters, and scoped document lists. Requires a document scope — namespace-wide grep times out. |
| `iris_info`                     | Namespace discovery: documents, jobs, CSP apps, metadata.                                                                                                      |
| `iris_macro`                    | Macro inspection: list, signature, definition, expand.                                                                                                         |
| `iris_table_info`               | Inspect a SQL table: class-projected vs. DDL, backing storage globals, optional row count.                                                                     |
| `resolve_dynamic_dispatch`      | Resolve `$classmethod`/`##class({var})` polymorphic dispatch to compiled candidate classes, with confidence scores.                                            |
| `extract_message_map_routing`   | Extract a compiled Ensemble `MessageMap` routing table (MessageType → Method) from a BusinessProcess/Router.                                                   |
| `find_subclass_implementations` | Find all concrete subclass implementations of a method across the full inheritance hierarchy.                                                                  |

---

## Debugging

| Tool           | What it does                                                                    |
| -------------- | ------------------------------------------------------------------------------- |
| `iris_debug`   | Map INT offsets to source lines, fetch error logs, capture error state.         |
| `iris_get_log` | Retrieve a full result by `log_id` when a tool returns `truncated: true`.       |
| `check_config` | Show active connection state — host, container, config file, write tool status. |

---

## Generation

| Tool                  | What it does                                                                  |
| --------------------- | ----------------------------------------------------------------------------- |
| `iris_generate`       | Build a context-rich prompt for generating ObjectScript. No API key required. |
| `iris_generate_class` | Generate and compile a class from a description (requires LLM API key).       |
| `iris_generate_test`  | Generate `%UnitTest` scaffolding for an existing class.                       |

---

## Interoperability

| Tool                        | What it does                                                                                                      |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `iris_production` ✦         | Start, stop, update, check, or recover a production.                                                              |
| `iris_interop_query` ✦      | Query production logs, queue depths, or message archive.                                                          |
| `iris_production_item` 🔒   | Enable, disable, or get/set settings on an individual production config item. Works via HTTP, no Docker required. |
| `iris_production_diff`      | Diff the running production config against the last source-controlled version.                                    |
| `iris_message_body`         | Read a message body by ID (plain-text or stream-backed). PHI-gated.                                               |
| `iris_business_rule_info`   | List or inspect Ensemble business rules (`Ens.Rule.RuleSet`).                                                     |
| `iris_credential_list`      | List Ensemble credentials (IDs/usernames only — passwords never returned).                                        |
| `iris_credential_manage` 🔒 | Create, update, or delete an Ensemble credential.                                                                 |
| `iris_lookup_manage`        | Read, write, delete, or list Ensemble lookup table entries (write actions gated).                                 |
| `iris_lookup_transfer`      | Export or import an Ensemble lookup table as XML (import gated).                                                  |

---

## Administration

| Tool                | What it does                                                                                              |
| ------------------- | --------------------------------------------------------------------------------------------------------- |
| `iris_admin`        | List namespaces, databases, users, roles, web apps; create/delete users (requires `IRIS_ADMIN_TOOLS=1`).  |
| `iris_containers` ✦ | List, select, or start IRIS Docker containers. Hot-swaps the active connection without a session restart. |

---

## Learning agent, skills, and knowledge base

| Tool                     | What it does                                                                                                                |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `agent_history`          | Recent tool-call history for the current session (tool, success, duration, timestamp).                                      |
| `agent_stats`            | Learning agent status: skill count, pattern count, KB size.                                                                 |
| `telemetry_query`        | Query the durable telemetry record beyond the in-memory session — by tool name, session id, or time range.                  |
| `telemetry_export_trace` | Export recorded tool calls as `{from, to, via, count, ts}` dispatch-trace records, aggregated.                              |
| `skill`                  | Manage the learning agent skill registry: list, describe, search, forget, or propose (mines recent calls into a new skill). |
| `skill_community`        | Browse or install community skills published to subscribed GitHub repos.                                                    |
| `kb`                     | Index markdown/text into the IRIS knowledge base, or recall content by keyword.                                             |

---

## Common error codes

| Code                    | Meaning                                                                          |
| ----------------------- | -------------------------------------------------------------------------------- |
| `POLICY_GATE`           | Call blocked by per-connection policy — see `allow` in `.iris-agentic-dev.toml`  |
| `SCOPE_REQUIRED`        | `iris_search` called without a document scope — pass a `documents` wildcard list |
| `STALE_CONTENT`         | `iris_doc` insert/delete_lines `expected` field didn't match stored content (anchor a blank line with the literal `<BLANK>`, never an empty string) |
| `CODE_EDIT_BLOCKED`     | Write to a `%` system class blocked by the code-edit gate                        |
| `CHECKIN_BLOCKED`       | SCM CheckIn called without `IRIS_SCM_ALLOW_CHECKIN=1`                            |
| `HTTP_EXECUTION_FAILED` | Atelier HTTP call failed — check host, port, credentials                         |
| `IRIS_UNREACHABLE`      | No IRIS connection discoverable — run `check_config`                             |
