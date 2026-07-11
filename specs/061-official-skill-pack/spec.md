# Feature Specification: Official InterSystems Skill Pack

**Feature Branch**: `061-official-skill-pack`
**Created**: 2026-07-09
**Status**: Draft
**Input**: User description: "An official, curated InterSystems skill pack for AI coding agents, distributed as real SKILL.md files written into the standard agent-skills directory convention (the same mechanism Claude Code, Cursor, and other agents already read natively) — not gated behind a live IRIS connection, so a brand-new developer with no IRIS installed anywhere still gets the pack. iris-agentic-dev gains a skill-install command that performs this file-based install directly (a thin, official wrapper analogous to 'claude plugin install stripe@claude-plugins-official'), and separately, existing ^SKILLS-based IRIS-side skill loading becomes an optional additional destination for teams who want skills to travel with their IRIS instance — not the only or required path. The pack covers iris-agentic-dev itself plus other InterSystems-ecosystem open-source projects (iris-vector-graph, ivr, iris-embedded-python-wrapper, etc.), so a developer can discover and install those projects by chatting with their agent."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install the official skill pack with no IRIS instance running (Priority: P1)

A developer who has never used InterSystems IRIS, and has no instance installed or running anywhere, wants their AI coding agent to know how to help them get started with IRIS and the broader InterSystems open-source ecosystem. They run a single install command. Afterward, their agent has working knowledge of IRIS/ObjectScript development, InterSystems open-source tools, and can guide the developer through getting an actual IRIS instance running for the first time — all without the install step itself requiring IRIS to exist.

**Why this priority**: This is the entire point of the pivot away from an IRIS-gated mechanism. If installing the skill pack still required a live IRIS connection, it would be useless for exactly the audience it's meant to reach — someone who doesn't have IRIS yet and needs the agent's help to get one running in the first place. Every other user story depends on this working.

**Independent Test**: On a machine with no IRIS instance installed, running, or ever configured, run the skill pack install command and verify skill content becomes available to the AI agent in the same session, with no error, prompt, or dependency related to an IRIS connection.

**Acceptance Scenarios**:

1. **Given** a developer machine with no IRIS instance installed anywhere, **When** the developer runs the skill pack install command, **Then** the install completes successfully and skill content is available to the agent.
2. **Given** the skill pack has been installed, **When** the developer asks their agent an IRIS-related question covered by the pack, **Then** the agent's response reflects the installed skill content.
3. **Given** the skill pack install command is run a second time (already installed), **When** the command completes, **Then** the pack is updated to the latest content rather than erroring out or duplicating entries.

---

### User Story 2 - Discover and install other InterSystems open-source projects via chat (Priority: P2)

A developer who has the official skill pack installed asks their agent about a capability outside iris-agentic-dev itself — for example, vector search or graph queries in IRIS. Their agent, informed by the skill pack, tells them about the relevant InterSystems open-source project (for example, iris-vector-graph), explains what it does, and can guide them through installing it.

**Why this priority**: This is what turns the skill pack from "documentation about one tool" into "a discovery surface for the whole ecosystem," which is the specific value the pack's author wants to provide — promoting the broader set of open-source projects, not just iris-agentic-dev. It depends on User Story 1 (the pack has to be installed and readable first).

**Independent Test**: With the skill pack installed and no other project-specific setup, ask the agent a question whose best answer is "use project X" for one of the bundled ecosystem projects, and verify the agent's response names that project and describes how to get it, without the developer having had any prior knowledge that the project existed.

**Acceptance Scenarios**:

1. **Given** the skill pack is installed, **When** a developer describes a need that a bundled ecosystem project addresses, **Then** the agent's response identifies that project by name and explains what it provides.
2. **Given** a developer wants to install one of the ecosystem projects the pack described, **When** they ask the agent to help install it, **Then** the agent can guide the developer through the actual installation for that project (which may have its own, project-specific installation mechanism distinct from the skill pack's).
3. **Given** an ecosystem project referenced by the pack has been renamed, deprecated, or replaced, **When** the pack is next updated, **Then** the pack's content reflects the current state rather than recommending a stale or dead project.

---

### User Story 3 - Skills travel with an IRIS instance for a team that wants that (Priority: P3)

A team that has adopted the official skill pack, and separately runs a shared IRIS instance across multiple developers, wants any skill content installed by one team member to also be available to teammates through that shared instance — without every developer needing to separately install the skill pack on their own machine.

**Why this priority**: This preserves the value of the existing IRIS-side skill loading mechanism for teams that already benefit from it, but it is explicitly optional and layered on top of the file-based mechanism (User Story 1) rather than a prerequisite for it. It is lower priority because the file-based install alone already delivers the core value; this is an additional convenience for a specific (team/shared-instance) audience.

**Independent Test**: With the skill pack installed via the file-based mechanism on one developer's machine and a shared IRIS instance available, mirror the pack's content to that instance, then verify a second developer's agent — connected to the same IRIS instance but without having run the file-based install locally — can access the same skill content.

**Acceptance Scenarios**:

1. **Given** a developer has installed the skill pack via the file-based mechanism and has access to a shared IRIS instance, **When** they choose to mirror the pack to that instance, **Then** the instance holds the same skill content the file-based install provided locally.
2. **Given** skill content has been mirrored to a shared IRIS instance, **When** a different developer's agent connects to that instance without a local file-based install, **Then** that developer's agent also has access to the mirrored skill content.
3. **Given** a developer has no access to any IRIS instance at all, **When** they use the skill pack, **Then** they experience the full value of User Stories 1 and 2 with no degradation, since the IRIS-side mirror is additive, not required.

---

### Edge Cases

- What happens when the standard agent-skills directory already contains a differently-named or user-authored skill that conflicts in content (not name) with something the official pack provides — for example, a personal skill with outdated IRIS guidance?
- What happens when a developer's AI coding agent does not support the standard skill-directory convention at all (an agent outside the open Agent Skills ecosystem)?
- What happens when the official pack references an ecosystem project (e.g., iris-vector-graph) whose own repository has moved, been renamed, or become unavailable at pack-update time?
- What happens when a developer installs the pack, then later that pack is updated with content that contradicts what the developer has since customized locally in the same skill files?
- What happens when a team mirrors the pack to a shared IRIS instance (User Story 3) and the file-based pack on individual developer machines is a different, older or newer, version than what's mirrored — which one does an agent prefer when both are reachable?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a single command that installs the official InterSystems skill pack by writing real skill files into the standard agent-skills directory convention already read natively by common AI coding agents.
- **FR-002**: The install command specified in FR-001 MUST succeed and make skill content available with no live IRIS connection, no IRIS instance configured, and no IRIS credentials of any kind.
- **FR-003**: System MUST make the installed skill pack's content available to the AI coding agent using the same mechanism the agent already uses for any other locally-installed skill — no iris-agentic-dev-specific tool call required to read installed skill content.
- **FR-004**: The skill pack MUST include content covering iris-agentic-dev itself and other InterSystems-ecosystem open-source projects designated for inclusion (at minimum iris-vector-graph, iris-vector-rag, and iris-embedded-python-wrapper).
- **FR-005**: Skill pack content covering an ecosystem project other than iris-agentic-dev MUST describe what that project does and enough about how to obtain/install it that a developer can act on the recommendation, even though that project's actual install mechanism may differ from the skill pack's own.
- **FR-006**: Re-running the install command MUST update existing official pack content to the latest version rather than erroring, duplicating files, or requiring a separate "update" command.
- **FR-007**: System MUST continue to support making skill content available through an existing IRIS instance (mirroring installed skill content into that instance) as an additional, explicitly-chosen destination — this MUST NOT be required for the file-based install in FR-001 to work.
- **FR-008**: System MUST NOT silently overwrite a locally-authored, non-official skill file that happens to occupy a name the official pack also uses; a naming collision MUST be visible to the developer rather than one version disappearing without explanation.
- **FR-009**: The skill pack's content about ecosystem projects MUST be revisable independently of an iris-agentic-dev software release, so that a stale or moved project reference can be corrected without shipping a new version of iris-agentic-dev itself.

### Key Entities

- **Official Skill Pack**: A curated, versioned collection of skill content covering iris-agentic-dev and designated InterSystems-ecosystem open-source projects, distributed as files written into the standard agent-skills directory convention. Analogous to a vendor-maintained plugin in the wider agent-skills ecosystem (e.g., a payment provider's official skill pack).
- **Skill File**: An individual unit of guidance within the pack, following the open agent-skills format (a folder containing a primary instructions file plus optional supporting scripts/references), readable natively by any compliant AI coding agent without going through iris-agentic-dev.
- **Ecosystem Project Reference**: Skill content that describes a specific InterSystems-ecosystem open-source project (not iris-agentic-dev itself) — what it does and how a developer would obtain and install it, serving as a discovery mechanism rather than an installer for that project.
- **IRIS-Side Mirror**: An optional copy of installed skill content pushed into a live IRIS instance so that content travels with that instance for other developers connecting to it, distinct from and layered on top of the primary file-based installation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer with no IRIS instance anywhere can go from "nothing installed" to "agent has working InterSystems ecosystem knowledge" in a single command, with zero IRIS-related errors or prerequisites encountered.
- **SC-002**: A developer asking their agent about a need covered by a bundled ecosystem project receives a response naming that specific project, rather than a generic or no answer, in the large majority of relevant questions.
- **SC-003**: Updating the official pack's content (including correcting a stale ecosystem project reference) does not require a new iris-agentic-dev release to reach developers who have already installed the pack.
- **SC-004**: Teams that choose to mirror the pack into a shared IRIS instance see no difference in the skill content available to a developer connecting via that instance compared to a developer who installed the pack locally.
- **SC-005**: A developer who already has personally-authored skills in their standard skills directory experiences no silent loss of that content when installing or updating the official pack.
