# PROJECT MAP: ryvos-new (Ultra-Lightweight Rust AI Agent Runtime)

## 1. Visi and Arsitektur Sistem
`ryvos-new` adalah Autonomous AI Coding Agent runtime paling ringan, cepat, dan aman:
- Bahasa Inti: Rust (Edition 2021 / Cargo Workspace).
- RAM Footprint: 15-30 MB idle, <60 MB full subsystems active.
- Delivery: Single static binary executable via `rust-embed`.
- Frontend Gateway: Svelte 5 + Tailwind CSS + Vite (Soft Blue Neumorphism, Anti-Slop R-01 to R-40, WCAG 2.2 AA).

## 2. Struktur Modul / Cargo Workspace (`crates/`)
| Crate | Peran and Batasan Modul |
| :--- | :--- |
| `crates/ryvos-core` | Data models, AppConfig (`config.toml`), Security Gate, error types. |
| `crates/ryvos-agent` | Autonomous agent execution loop, active run cancellation, decision engine, context compactor, stall guardian. |
| `crates/ryvos-llm` | Multi-provider streaming adapter (OpenAI, Anthropic, Gemini, Ollama, DeepSeek), KeyRotator pool, hardened SSE parser. |
| `crates/ryvos-tools` | Built-in native tools (read/edit/write file, run bash, git operations, egress SSRF protection). |
| `crates/ryvos-mcp` | Model Context Protocol client (stdio and SSE transport). |
| `crates/ryvos-gateway` | Axum HTTP + WebSocket gateway server (unblocked reading loop) + Embedded Web UI (`ui-src/` Svelte 5). |
| `crates/ryvos-channels` | Channel adapters (Telegram bot, Discord, WhatsApp webhooks). |
| `crates/ryvos-memory` | SQLite persistent sessions (`SessionMetaStore`), failure journal, audit log, OpenViking memory bridge. |
| `crates/ryvos-skills` | On-demand skill discovery, tool extension registry. |
| `crates/ryvos-tui` | Terminal text user interface. |

## 3. Direktori Kunci
- `crates/ryvos-gateway/ui-src/`: Kode sumber Frontend Svelte 5 (Vite + Tailwind, Soft Blue Neumorphism).
- `crates/ryvos-gateway/ui/`: Hasil kompilasi static dist yang di-embed langsung ke Rust binary.
- `src/main.rs`: Root CLI entrypoint (`ryvos repl`, `run`, `serve`, `doctor`), wiring `SessionMetaStore`, `safety_memory`, `failure_journal`.
- `.agents/rules/`: Aturan baku guardrails untuk agen pengembangan Antigravity cluster.
