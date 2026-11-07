---
name: ryvos-codebase-map
description: Architectural map of ryvos-new crates, Svelte 5 web UI, build commands, and memory invariants. Use when exploring or contributing to the ryvos-new codebase. Peta arsitektur crate, frontend Svelte, dan panduan kode ryvos-new.
---
# ryvos-new Codebase Map

## When to use / not use
- **Use when**: Navigating, extending, or refactoring the `ryvos-new` workspace (`/opt/data/home/ryvos-new`).
- **Do not use for**: General external Rust projects not sharing the ryvos architecture.

## Core rules
1. **Architecture & Crates (`crates/`)**:
   - `ryvos-core`: Data models, `AppConfig`, error types, SecurityGate contracts.
   - `ryvos-agent`: Autonomous execution loop, context compaction, Guardian stall detection, Judge verification.
   - `ryvos-llm`: Multi-provider streaming client (OpenAI, Anthropic, Gemini, Ollama) and SSE decoder.
   - `ryvos-tools`: Built-in native tools (file IO, bash execution, git operations, safety guards).
   - `ryvos-mcp`: Model Context Protocol client implementation (stdio and SSE transport).
   - `ryvos-gateway`: Axum HTTP and WebSocket gateway server + Embedded Web UI.
   - `ryvos-channels`: Messaging adapters (Telegram, Discord, WhatsApp webhooks).
   - `ryvos-memory`: SQLite persistence, session storage, failure journal, audit log.
   - `ryvos-skills`: On-demand progressive disclosure skill discovery and execution engine.
   - `ryvos-tui`: Terminal text user interface.
2. **Frontend UI Integration**:
   - Source: `crates/ryvos-gateway/ui-src` (Svelte 5 + Tailwind CSS + Vite).
   - Embedded Output: `crates/ryvos-gateway/ui` (embedded into binary via `rust-embed`).
   - Build command: `NODE_OPTIONS="--max-old-space-size=1024" npm run build` inside `ui-src`.
3. **Compilation & Resource Invariants**:
   - Concurrency limits: Always run `cargo build -j 2` and `cargo test -j 2` to respect VPS 4GB RAM ceiling.
   - RAM footprint: 15–30 MB idle, <60 MB active.
   - Single binary: Final executable embeds all web assets without runtime external files.
4. **Code Quality Standards**:
   - NOKOMEN: Zero slop AI comments in Rust/JS code. Clean self-documenting code.
   - Check commands: `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace -j 2`.

## Procedure
1. Read relevant crate `Cargo.toml` and `src/lib.rs` to inspect module contracts.
2. Implement feature in targeted crate following domain boundaries.
3. If modifying web UI, test in `ui-src`, build, and verify static embed.
4. Run standard test and clippy commands with `-j 2`.

## Rust implementation notes
```rust
// Standard workspace invocation contract
// Entrypoint: src/main.rs -> ryvos_gateway::run_server / ryvos_agent::run_agent
```

## Pitfalls
- Running unconstrained `cargo build` on VPS triggering OOM killer.
- Modifying `crates/ryvos-gateway/ui` directly instead of compiling from `ui-src`.

## Done checklist
- [ ] Code compiles with `cargo check --workspace`.
- [ ] Clippy passes with zero warnings: `cargo clippy --workspace -- -D warnings`.
- [ ] All tests pass with `-j 2`: `cargo test --workspace -j 2`.
- [ ] RAM footprint verified within 15–30 MB idle range.
