# 🗺️ PROJECT MAP: ryvos-new (Ultra-Lightweight Rust AI Agent Runtime)

## 1. Visi & Arsitektur Sistem
`ryvos-new` adalah fork evolusioner dari Ryvos yang dirancang khusus untuk menjadi **Autonomous AI Coding Agent runtime paling ringan, cepat, dan aman**:
- **Bahasa Inti**: Rust (Edition 2021/Cargo Workspace).
- **RAM Footprint**: 15–30 MB idle, <60 MB full subsystems active.
- **Delivery**: Single static binary executable.
- **Frontend Gateway**: Svelte 5 + Tailwind CSS + Vite (embedded via `rust-embed` ke Axum).

## 2. Struktur Modul / Cargo Workspace (`crates/`)
| Crate | Peran & Batasan Modul |
| :--- | :--- |
| `crates/ryvos-core` | Data models, AppConfig (`ryvos.toml`), Security Gate & Guardrails, error types. |
| `crates/ryvos-agent` | Autonomous agent execution loop, decision engine, context window compactor, guardian stall-watcher. |
| `crates/ryvos-llm` | Multi-provider streaming adapter (OpenAI-compatible, Anthropic, Gemini, Ollama, DeepSeek). |
| `crates/ryvos-tools` | Built-in native tools (read/edit/write file, run bash, git operations, safety checks). |
| `crates/ryvos-mcp` | Model Context Protocol client (stdio & SSE transport). |
| `crates/ryvos-gateway` | Axum HTTP + WebSocket gateway server + Embedded Web UI (`ui-src/` Svelte 5). |
| `crates/ryvos-channels` | Channel adapters (Telegram bot, Discord, WhatsApp webhooks). |
| `crates/ryvos-memory` | SQLite persistent sessions, failure journal, audit log, OpenViking memory bridge. |
| `crates/ryvos-skills` | On-demand skill discovery, tool extension registry. |
| `crates/ryvos-tui` | Terminal text user interface. |

## 3. Direktori Kunci
- `crates/ryvos-gateway/ui-src/`: Kode sumber Frontend Svelte 5 (Vite + Tailwind).
- `crates/ryvos-gateway/ui/`: Hasil kompilasi static dist yang di-embed langsung ke Rust binary.
- `src/main.rs`: Root CLI entrypoint (`ryvos repl`, `run`, `serve`, `doctor`).
- `.agents/rules/`: Aturan baku guardrails untuk agen pengembangan Antigravity cluster.
