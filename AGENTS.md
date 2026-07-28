# AGENTS.md

Guidance for AI coding agents working in this repository.

## Scope and Maintenance Policy

This document is intentionally **stable** and should avoid implementation snapshots.

- Keep this file focused on durable engineering rules and workflows.
- Do not duplicate detailed API/type/feature inventories here.
- Treat the following as the source of truth:
  - architecture and data flow: [Concept.md](./Concept.md)
  - user-facing usage and examples: [README.md](./README.md), `examples/*`
  - current feature flags and dependencies: each crate's `Cargo.toml`

If implementation changes, prefer updating those source-of-truth files instead of
adding drift-prone details to `AGENTS.md`.

## Project Overview

**promkit** is a Rust toolkit for interactive command-line prompts and terminal UIs,
built on top of [crossterm](https://github.com/crossterm-rs/crossterm).

## Workspace Map (High Level)

- `promkit/`: optional `Prompt` lifecycle runtime, capabilities, and widget facade
- `promkit-core/`: rendering primitives and terminal drawing
- `promkit-widgets/`: reusable widget states
- `promkit-derive/`: proc macros
- `tests/readline/`: readline terminal rendering regression test crate and scenarios
- `examples/`: runnable examples

## Architectural Rules

The authoritative model is in [Concept.md](./Concept.md). Keep these boundaries:

1. The `Prompt` lifecycle and terminal event stream belong to `promkit`.
2. Key bindings, focus transitions, and other event policy belong to application
   `Prompt` implementations, including the examples.
3. `promkit-widgets` is state-to-view projection, without event-loop policy.
4. Rendering concerns belong to `promkit-core`.

When adding features, preserve these boundaries before optimizing code layout.

## Coding Conventions

### Feature Wiring

- Control module exposure via feature flags.
  - Evidence: [promkit-widgets/src/lib.rs](./promkit-widgets/src/lib.rs), [promkit/src/lib.rs](./promkit/src/lib.rs)
- Wire `promkit` features to `promkit-widgets` features.
  - Evidence: [promkit/Cargo.toml](./promkit/Cargo.toml), [promkit-widgets/Cargo.toml](./promkit-widgets/Cargo.toml)

## Change Workflow for Agents

When implementing a change:

1. Locate the boundary first (application vs `promkit` vs `promkit-widgets` vs
   `promkit-core`).
2. Make the smallest coherent edit set.
3. Update tests/examples/docs that demonstrate behavior.
4. Run validation commands locally when possible.
5. For non-trivial implementation work, run the relevant `cargo bench` target
   before and after the change and check for performance regressions.

Recommended commands:

```bash
cargo fmt --all -- --check
cargo clippy
cargo test -- --nocapture --format pretty
```

For broader changes, also build example crates.

### Performance Validation

- Benchmark new widgets, data structures, rendering paths, algorithms, and other
  substantial implementations.
- Compare against a pre-change result, using Criterion named baselines when the
  relevant benchmark supports them.
- Investigate and report meaningful regressions instead of relying only on a
  successful benchmark run.
- Documentation-only edits, formatting, and small changes outside
  performance-sensitive paths may omit benchmarks. Small changes to hot paths
  still require benchmark comparison.

Select the benchmark and feature flags documented by the relevant crate. A
typical invocation is:

```bash
cargo bench -p <crate> --bench <target> --features <features>
```

## What Not to Put Here

- Exhaustive feature tables copied from `Cargo.toml`
- Fine-grained type internals likely to change
- Temporary implementation notes or one-off bug memories

If such details are needed, place them near the code, tests, or in `Concept.md`.
