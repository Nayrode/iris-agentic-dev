# Contributing

Contributions are welcome — bug reports, feature requests, skills, and code.

## Quick start

```bash
git clone https://github.com/intersystems-community/iris-agentic-dev
cd iris-agentic-dev
cargo build
cargo test
```

See [CLAUDE.md](CLAUDE.md) for the full dev setup including the local IRIS container.

## What to work on

Issues labeled [`good first issue`][gfi] are scoped and self-contained.
Issues labeled [`help wanted`][hw] are higher impact but may need more context —
comment before starting so we can align on approach.

[gfi]: https://github.com/intersystems-community/iris-agentic-dev/labels/good%20first%20issue
[hw]: https://github.com/intersystems-community/iris-agentic-dev/labels/help%20wanted

## Pull requests

- Open an issue first for anything non-trivial so the approach can be agreed before code is written.
- Tests are required. See the testing philosophy in [CLAUDE.md](CLAUDE.md) — IRIS-touching
  code must be tested against a live container, not mocks.
- `cargo fmt --all` and `cargo clippy -- -D warnings` must pass before submitting.
- Keep PRs focused. One logical change per PR.

## Contributing a skill

Skills live in `skills/` (full) and `light-skills/skills/` (trimmed for token budget).
Each skill is a directory with a `SKILL.md` and optional `references/` subdirectory.

A skill is worth adding if it:

- Corrects a pattern where AI models reliably make mistakes with IRIS/ObjectScript
- Encodes institutional knowledge that isn't in public documentation
- Has a measurable effect — run the benchmark harness in `benchmark/021/` before and after

See [`light-skills/skills/objectscript-review/`](light-skills/skills/objectscript-review/)
for a reference example.

## Bug reports

Include:

- `iris-agentic-dev --version`
- IRIS version and edition (Community / Enterprise / HealthShare)
- Deployment (native Windows/Linux, Docker, VS Code extension)
- The exact tool call that failed and the full error output
- Whether the issue is reproducible or intermittent

## License

By contributing you agree that your contributions will be licensed under the
[MIT License](LICENSE).
