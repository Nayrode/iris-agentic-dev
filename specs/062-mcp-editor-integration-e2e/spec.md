# Feature Specification: MCP Editor Integration E2E

**Feature Branch**: `062-mcp-editor-integration-e2e`
**Created**: 2026-07-09
**Status**: Draft
**Input**: User description: "Real end-to-end verification that iris-agentic-dev registers and connects correctly as an MCP server inside GitHub Copilot (VS Code) first, Claude Code second, and OpenCode third — proving each editor resolves the correct pinned IRIS instance from its own native MCP config mechanism, not a mocked or bypassed one. Motivated by a live bug: Claude Code's ~/.claude.json had an MCP server entry unpinned to any IRIS container, causing iris-agentic-dev to fall through its full auto-discovery cascade and connect to the wrong instance nondeterministically."

## Problem Statement

iris-agentic-dev already has real E2E coverage for OpenCode (`039-skills-e2e`): it spawns a real OpenCode process, injects config via `OPENCODE_CONFIG_CONTENT`, and asserts against a live IRIS container. It has no equivalent for the two higher-priority environments:

- **GitHub Copilot (VS Code)** — `benchmark/021/runner/copilot.py` is a stub (`NotImplementedError`) pointing at a plan (`specs/021-path-aware-benchmark/`) that no longer exists on disk. Zero real coverage.
- **Claude Code** — `benchmark/021/runner/claude_code.py` drives the raw Anthropic API against an MCP subprocess directly. It never exercises Claude Code the application, never touches `~/.claude.json` or per-project `mcpServers` config, and is currently broken: it spawns the binary by its pre-rename name (`iris-dev`), which no longer exists (only `iris-agentic-dev` is installed).

None of the three existing harnesses test the actual failure that occurred: an editor's own MCP config resolving to the wrong (or no) pinned IRIS instance. All of them assume the MCP server config is already correct and test only tool-call behavior after that point. This is the gap that let a real, live-broken config (an entry with no `IRIS_CONTAINER`) go undetected — it fell through the full discovery cascade silently instead of failing loud or connecting deterministically.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - GitHub Copilot (VS Code) resolves the correct pinned IRIS instance (Priority: P1)

A developer adds an `iris-agentic-dev` MCP server entry to VS Code's `mcp.json` (workspace or user settings), pinned to a specific IRIS container via `IRIS_CONTAINER`. They open a Copilot Chat session and ask it to run a tool that reports the connected IRIS instance (e.g. `check_config`). The reported instance matches the pinned container, not some other instance that happened to be discovered first.

**Why this priority**: Copilot in VS Code is the primary environment. If this path is silently wrong, it's the highest-blast-radius version of the bug that was just found in Claude Code's config, and it's currently completely unverified — the existing harness is a stub.

**Independent Test**: Start two distinct IRIS containers. Configure VS Code's MCP config to pin `iris-agentic-dev` to one specific container. Drive a real Copilot Chat session (via the VS Code test/automation surface, not a mocked client) that invokes a tool reporting connection identity. Assert the reported container matches the pinned one, not the other.

**Acceptance Scenarios**:

1. **Given** two live IRIS containers and a Copilot MCP config pinning `iris-agentic-dev` to container A via `IRIS_CONTAINER`, **When** a Copilot Chat session calls a tool that reports connection identity, **Then** the reported instance is container A.
2. **Given** the same setup but the pin is changed to container B and the MCP server is restarted, **When** the same tool is called, **Then** the reported instance is container B.
3. **Given** an MCP config entry with no `IRIS_CONTAINER` pin and multiple candidate IRIS instances reachable, **When** the tool is called twice in separate sessions, **Then** the harness surfaces whatever nondeterminism exists (same or different instance) rather than assuming it's stable — this is the regression class being guarded against.

---

### User Story 2 - Claude Code resolves the correct pinned IRIS instance (Priority: P2)

The same scenario as User Story 1, but driven through Claude Code's own MCP configuration mechanism (`~/.claude.json` `mcpServers`, or project-scoped overrides) and a real Claude Code session — not the Anthropic-API-plus-raw-MCP-subprocess bypass the current `benchmark/021/runner/claude_code.py` uses.

**Why this priority**: This is the environment where the actual live bug was found. It's second priority because Copilot is the stated primary environment, but this scenario must exist and must exercise Claude Code's real config-loading path, since that's exactly where the config drift went undetected.

**Independent Test**: Configure a `~/.claude.json`-equivalent (isolated, temp-directory-scoped, not the developer's real global config) with an `iris-agentic-dev` entry pinned to a specific container. Drive a real Claude Code session against that isolated config. Assert the connection identity tool reports the pinned container.

**Acceptance Scenarios**:

1. **Given** an isolated Claude Code config with `iris-agentic-dev` pinned to a specific IRIS container, **When** a real Claude Code session calls the connection-identity tool, **Then** the reported instance matches the pin.
2. **Given** an isolated Claude Code config with an entry that has no `IRIS_CONTAINER` pin (reproducing the exact shape of the bug that was found), **When** the same tool is called, **Then** the harness detects and reports the absence of a deterministic pin as a failure condition, not a silent pass.
3. **Given** multiple `mcpServers` entries in the same config pointing at the same `iris-agentic-dev` binary under different names (reproducing the exact multi-entry confusion found live), **When** each entry is exercised independently, **Then** each resolves to its own correctly pinned instance with no cross-contamination between entries.

---

### User Story 3 - OpenCode continues to resolve the correct pinned IRIS instance (Priority: P3)

OpenCode already has a real E2E harness (`039-skills-e2e`) that configures MCP via `OPENCODE_CONFIG_CONTENT` and asserts tool calls succeed against a live IRIS container. This story extends that existing coverage with the same connection-identity assertion used in User Stories 1 and 2, so all three environments are checked for the same failure mode using a comparable assertion.

**Why this priority**: Lowest priority because real E2E coverage already exists for OpenCode; this is an incremental strengthening of an existing harness rather than building new infrastructure from scratch.

**Independent Test**: Extend the existing OpenCode harness's IRIS-configured scenario (US2 of `039-skills-e2e`) with an assertion that the reported connection identity matches the container the harness itself started and pinned via `IRIS_CONTAINER`.

**Acceptance Scenarios**:

1. **Given** the existing OpenCode harness's live-IRIS scenario, **When** the connection-identity tool is called in addition to the existing `iris_compile` assertion, **Then** the reported instance matches the harness-managed container.

---

### Edge Cases

- What happens when the editor's MCP config mechanism itself fails to launch the server at all (crash, wrong binary path, wrong binary name after a rename)? The harness must distinguish "wrong instance" from "no instance" and report each distinctly.
- What happens when two configured entries in the same editor's config reference the same underlying binary but different `IRIS_CONTAINER` pins, and are invoked in the same session? Cross-contamination between them must be detectable.
- What happens when the pinned container doesn't exist or isn't running at session start? The harness must capture the editor's actual observed behavior (error surfaced to the user vs. silent fallback to discovery) rather than assume one.
- What happens when running Copilot/Claude Code E2E on a CI runner with no GUI? Both editors' automation surfaces must have a documented headless invocation path before a harness can be built on them; if no such path exists for one of them, that must be surfaced as a blocking finding, not worked around silently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST have a real, non-stub E2E harness that drives an actual GitHub Copilot Chat session in VS Code (or its documented headless/automation equivalent) against a live IRIS container, and asserts the reported connection identity matches a deliberately pinned container.
- **FR-002**: The project MUST have a real, non-stub E2E harness that drives an actual Claude Code session (not an Anthropic-API-plus-bypassed-MCP-subprocess substitute) against a live IRIS container, using Claude Code's own MCP configuration file format, and asserts the reported connection identity matches a deliberately pinned container.
- **FR-003**: The existing OpenCode E2E harness (`039-skills-e2e`) MUST be extended with the same connection-identity assertion used in FR-001 and FR-002, so all three environments are checked for the same failure mode with a comparable assertion.
- **FR-004**: Each harness MUST run against at least two distinct, simultaneously-live IRIS containers per test run, so that "resolved to the correct one" is a meaningful assertion rather than trivially true with only one candidate.
- **FR-005**: Each harness MUST include a scenario that reproduces the exact unpinned-config shape of the live bug that motivated this feature (an MCP server entry with no `IRIS_CONTAINER` and multiple reachable candidates) and asserts the harness treats non-deterministic or wrong-instance resolution as a failure, not a pass.
- **FR-006**: The Claude Code harness MUST include a scenario with multiple `mcpServers` entries pointing at the same binary under different names (reproducing the exact multi-entry confusion found in the live bug), asserting no cross-contamination of resolved IRIS instance between entries.
- **FR-007**: The stale, broken `benchmark/021/runner/claude_code.py` and `benchmark/021/runner/copilot.py` MUST be replaced or fixed as part of this feature — not left in place alongside new, separate harnesses — so there is one authoritative E2E path per environment, not two divergent ones.
- **FR-008**: Every harness introduced or extended by this feature MUST run against a live IRIS container and a real invocation of the target editor/tool's own process — no assertion may be satisfied by a mock of the editor, a mock of IRIS, or a bypass of the editor's native MCP config-loading mechanism.
- **FR-009**: If GitHub Copilot's automation surface has no viable headless/CI invocation path at the time of planning, this MUST be surfaced explicitly as a blocking finding before implementation proceeds, rather than substituting a lower-fidelity stand-in silently.

### Key Entities

- **ConnectionIdentityTool**: An MCP tool call (e.g. `check_config`) whose result reveals which physical IRIS instance the server is actually connected to, used as the common assertion surface across all three harnesses.
- **PinnedContainerScenario**: A test fixture with two or more live IRIS containers and an editor config that pins the MCP server to one of them by name.
- **UnpinnedConfigScenario**: A test fixture reproducing the exact bug shape found live — an MCP server entry with no explicit container pin and multiple reachable candidates — used to assert the failure mode is caught, not silently passed.
- **MultiEntryScenario**: A test fixture (Claude Code specific) with multiple named MCP server entries pointing at the same binary, each pinned differently, asserting no cross-contamination.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Given two live IRIS containers, each of the three editor E2E harnesses (Copilot, Claude Code, OpenCode) correctly identifies which one its MCP server session is actually connected to, in every run.
- **SC-002**: The unpinned-config scenario (reproducing the live bug) is caught as a failure by at least the Claude Code harness before this feature is complete, and by the Copilot harness where the editor's config format allows the same shape.
- **SC-003**: Neither `benchmark/021/runner/claude_code.py` nor `benchmark/021/runner/copilot.py` remains in its current broken/stub state after this feature ships — each either becomes the real harness or is removed in favor of one.
- **SC-004**: A developer introducing a future MCP config regression (wrong binary name, missing pin, duplicate unpinned entries) would have at least one of these three harnesses fail in CI, rather than surviving to be discovered live as this one was.
- **SC-005**: Given no viable headless automation path exists for one of the three editors, that gap is documented as an explicit, named limitation in this feature's output — not silently absent from the final harness set.

## Assumptions

- GitHub Copilot Chat in VS Code has some form of scriptable/headless invocation surface (e.g. via VS Code's extension test harness or Copilot's own CLI/API if one exists); this needs confirming during planning per FR-009, not assumed true here.
- Claude Code supports a fully isolated, temp-directory-scoped config invocation (analogous to OpenCode's `OPENCODE_CONFIG_CONTENT`/`OPENCODE_DB`) so tests don't touch the developer's real `~/.claude.json`.
- At least two IRIS containers can be run simultaneously in CI without resource contention severe enough to make results flaky.
- The existing `039-skills-e2e` harness's structure (isolated env, real process, live IRIS, structured JSON result) is the right pattern to extend to Copilot and Claude Code, not a pattern to replace.

## Out of Scope

- Testing tool-call correctness/quality (e.g. skill lift, compile behavior) for Copilot or Claude Code — that's the existing benchmark harness's job; this feature is scoped strictly to MCP config resolution and connection identity.
- Windows or WSL2 paths for any of the three harnesses — Mac/Linux only, consistent with `039-skills-e2e`.
- Editors or tools outside Copilot, Claude Code, and OpenCode.
- Fixing the underlying discovery-cascade behavior itself (whether unpinned configs *should* fail loud vs. fall through) — that's a product decision for `crates/iris-agentic-dev-core/src/iris/discovery.rs`, out of scope for this E2E-verification feature.
