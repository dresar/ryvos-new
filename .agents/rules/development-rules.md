---
description: Aturan baku arsitektur, kualitas kode, pembatasan resource memori, dan konvensi pengembangan ryvos-new.
globs: "**/*.rs, **/*.toml, **/*.svelte, **/*.js"
---

# 🛡️ Aturan Baku Pengembangan ryvos-new

Aturan ini wajib ditaati oleh seluruh agen (`agy12` dan cluster worker) saat memodifikasi atau memperluas kode pada repositori `ryvos-new`.

## 1. Prinsip Efisiensi Ekstrem & Zero-Bloat
1. **Target Memory Invariant**: Idle RAM WAJIB tetap berada di kisaran 15–30 MB, dan <60 MB saat semua subsistem aktif. Dilarang menambah dependensi Rust yang menarik runtime berat atau runtime JavaScript/V8 ke binary utama.
2. **Standard Library & Tokio Idiomatic**: Utamakan primitives standar Rust (`std::sync`, `tokio::sync`) sebelum mempertimbangkan crate baru.
3. **No Slop Comments**: Dilarang menulis komentar penjelasan generik AI pada kode sumber Rust (`// this function adds...`). Kode harus self-documenting dan ringkas.
4. **Single-Binary Invariant**: Seluruh perubahan UI di `crates/ryvos-gateway/ui-src` wajib dikompilasi ke `crates/ryvos-gateway/ui` dan di-embed ke binary via `rust-embed`.

## 2. Resource-Aware Build Guardrails (VPS 4GB RAM)
1. **Cargo Build Concurrency**:
   - Dilarang menjalankan `cargo build` dengan thread paralel penuh.
   - Wajib gunakan flag pembatas core: `cargo build -j 2` atau `cargo test -j 2` guna menghindari trigger Linux OOM Killer pada cgroup 4GB.
2. **Frontend Build**:
   - Kompilasi `ui-src` wajib menyertakan flag pembatas:
     ```bash
     NODE_OPTIONS="--max-old-space-size=1024" npm run build
     ```

## 3. Tool & Protocol Contracts
1. **OpenAI & Anthropic Compatibility**: Parser streaming SSE wajib toleran terhadap anomali format (seperti model tanpa leading slash atau tanpa `/chat/completions`).
2. **Safe Tool Execution**: Tool bash harus selalu menerapkan timeout, per-command process isolation, dan filter pola destruktif (`rm -rf /`, raw disk format).
3. **Graceful Degradation**: Jika memory provider (OpenViking / SQLite) tidak tersedia, agent loop wajib tetap berjalan dalam mode in-memory tanpa panik (*zero unhandled panics*).
