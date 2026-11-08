"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { ProviderKeyInfo } from "@/types/api";

export default function KeysPage() {
  const [providers, setProviders] = useState<ProviderKeyInfo[]>([]);
  const [activeProvider, setActiveProvider] = useState("");
  const [activeModelId, setActiveModelId] = useState("");
  const [activeKeysCount, setActiveKeysCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const [inputKeys, setInputKeys] = useState<Record<string, string>>({});
  const [inputBaseUrls, setInputBaseUrls] = useState<Record<string, string>>({});
  const [selectedModels, setSelectedModels] = useState<Record<string, string>>({});
  const [showKeyText, setShowKeyText] = useState<Record<string, boolean>>({});
  const [savingProvider, setSavingProvider] = useState<string>("");

  const loadData = async () => {
    setLoading(true);
    setError("");
    try {
      const res = await api.keys.list();
      setProviders(res.providers || []);
      setActiveProvider(res.active_provider || "");
      setActiveModelId(res.active_model_id || "");
      setActiveKeysCount(res.active_keys_count || 0);

      const initModels: Record<string, string> = {};
      const initBaseUrls: Record<string, string> = {};
      (res.providers || []).forEach((p) => {
        initModels[p.id] = p.default_model || (p.models[0]?.id ?? "");
        initBaseUrls[p.id] = p.base_url || "";
      });
      setSelectedModels(initModels);
      setInputBaseUrls(initBaseUrls);
    } catch (err: any) {
      setError(err?.message || "Gagal memuat konfigurasi API key");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleSaveKeys = async (providerId: string, activate = false) => {
    setSavingProvider(providerId);
    setError("");
    setNotice("");
    try {
      const rawText = inputKeys[providerId] || "";
      const parsedKeys = rawText
        .split(/[\n,;]/)
        .map((k) => k.trim())
        .filter((k) => k.length > 0);

      const res = await api.keys.put({
        provider: providerId,
        api_keys: parsedKeys.length > 0 ? parsedKeys : undefined,
        base_url: inputBaseUrls[providerId] || undefined,
        default_model: selectedModels[providerId] || undefined,
        activate,
      });

      if (res.ok) {
        setNotice(
          `Kunci provider ${providerId.toUpperCase()} berhasil disimpan${
            activate ? " dan diaktifkan sebagai model utama" : ""
          }!`
        );
        setInputKeys((prev) => ({ ...prev, [providerId]: "" }));
        await loadData();
      }
    } catch (err: any) {
      setError(err?.message || `Gagal menyimpan kunci ${providerId}`);
    } finally {
      setSavingProvider("");
    }
  };

  const handleDeleteKeys = async (providerId: string) => {
    if (!confirm(`Hapus seluruh kunci untuk provider ${providerId.toUpperCase()}?`)) {
      return;
    }
    setSavingProvider(providerId);
    setError("");
    setNotice("");
    try {
      const res = await api.keys.delete(providerId);
      if (res.ok) {
        setNotice(`Kunci provider ${providerId.toUpperCase()} telah dihapus.`);
        await loadData();
      }
    } catch (err: any) {
      setError(err?.message || `Gagal menghapus kunci ${providerId}`);
    } finally {
      setSavingProvider("");
    }
  };

  const activeProvidersCount = providers.filter((p) => p.has_key).length;
  const totalKeysPool = providers.reduce((acc, p) => acc + (p.key_count || 0), 0);

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold tracking-tight text-text">
            Manajemen API Key dan Provider AI
          </h2>
          <p className="text-sm text-text-secondary mt-1">
            Kelola kunci API, rotasi key pool, dan aktivasi provider. Hanya model dari provider ber-API Key yang diizinkan aktif.
          </p>
        </div>
        <Link
          href="/chat"
          className="btn-primary inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold self-start sm:self-auto shadow-raised-sm"
        >
          <Icons.Chat className="w-4 h-4" />
          <span>Buka Workspace Chat</span>
        </Link>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="neu-card p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-soft text-primary flex items-center justify-center shadow-raised-sm">
            <Icons.Key className="w-5 h-5" />
          </div>
          <div>
            <span className="text-xs text-text-secondary font-medium">Provider Aktif</span>
            <p className="text-lg font-bold text-text tabular-nums">
              {activeProvidersCount} <span className="text-xs text-text-secondary font-normal">/ {providers.length} Didukung</span>
            </p>
          </div>
        </div>

        <div className="neu-card p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-soft text-primary flex items-center justify-center shadow-raised-sm">
            <Icons.Runs className="w-5 h-5" />
          </div>
          <div>
            <span className="text-xs text-text-secondary font-medium">Total Key Pool</span>
            <p className="text-lg font-bold text-text tabular-nums">
              {totalKeysPool} <span className="text-xs text-text-secondary font-normal">Kunci Rotasi</span>
            </p>
          </div>
        </div>

        <div className="neu-card p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-soft text-primary flex items-center justify-center shadow-raised-sm">
            <Icons.Cpu className="w-5 h-5" />
          </div>
          <div className="truncate">
            <span className="text-xs text-text-secondary font-medium">Runtime Aktif</span>
            <p className="text-sm font-bold text-primary truncate font-mono">
              {activeProvider ? `${activeProvider} (${activeModelId})` : "Belum Dipilih"}
            </p>
          </div>
        </div>
      </div>

      {notice && (
        <div className="p-4 rounded-xl bg-green-500/10 border border-green-500/20 text-sm text-text flex items-center justify-between">
          <span className="font-semibold text-green-700 dark:text-green-400">{notice}</span>
          <button
            type="button"
            onClick={() => setNotice("")}
            className="text-xs font-bold uppercase text-text-secondary hover:text-text px-2 py-1"
          >
            Tutup
          </button>
        </div>
      )}

      {error && (
        <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-sm text-danger flex items-center justify-between">
          <span>{error}</span>
          <button
            type="button"
            onClick={() => setError("")}
            className="text-xs font-bold uppercase text-text-secondary hover:text-text px-2 py-1"
          >
            Tutup
          </button>
        </div>
      )}

      {loading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="neu-card p-6 min-h-[200px] animate-pulse" />
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {providers.map((prov) => {
            const isSaving = savingProvider === prov.id;
            const hasExisting = prov.has_key;
            const showKey = showKeyText[prov.id] || false;

            return (
              <div
                key={prov.id}
                className={`neu-card p-5 sm:p-6 flex flex-col justify-between gap-4 transition-all ${
                  prov.is_active
                    ? "border-2 border-primary shadow-floating"
                    : "border border-border/60"
                }`}
              >
                <div className="space-y-3">
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2.5">
                      <div className="w-8 h-8 rounded-lg bg-surface-inset flex items-center justify-center text-primary font-bold shadow-raised-sm">
                        <Icons.Key className="w-4 h-4" />
                      </div>
                      <div>
                        <h3 className="text-base font-bold text-text">{prov.name}</h3>
                        <span className="text-[10px] font-mono text-text-secondary uppercase">
                          ID: {prov.id}
                        </span>
                      </div>
                    </div>

                    <div className="flex items-center gap-1.5 flex-wrap justify-end">
                      {prov.is_active && (
                        <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold bg-primary text-white shadow-raised-sm">
                          Model Utama
                        </span>
                      )}
                      <span
                        className={`px-2.5 py-0.5 rounded-full text-[11px] font-semibold ${
                          hasExisting
                            ? "bg-green-500/10 text-green-700 dark:text-green-400 border border-green-500/20"
                            : "bg-surface-inset text-text-secondary border border-border/60"
                        }`}
                      >
                        {hasExisting ? `Aktif (${prov.key_count} Key)` : "Belum Ada Key"}
                      </span>
                    </div>
                  </div>

                  <div>
                    <label
                      htmlFor={`model-select-${prov.id}`}
                      className="block text-xs font-semibold text-text-secondary mb-1"
                    >
                      Pilihan Model Default
                    </label>
                    <select
                      id={`model-select-${prov.id}`}
                      value={selectedModels[prov.id] || ""}
                      onChange={(e) =>
                        setSelectedModels((prev) => ({ ...prev, [prov.id]: e.target.value }))
                      }
                      className="neu-inset w-full px-3 py-2 text-xs font-mono rounded-xl bg-surface text-text border border-border/60 focus:outline-none"
                    >
                      {prov.models.map((m) => (
                        <option key={m.id} value={m.id}>
                          {m.name} ({m.id})
                        </option>
                      ))}
                    </select>
                  </div>

                  {hasExisting && prov.masked_keys.length > 0 && (
                    <div className="p-2.5 rounded-xl bg-surface-inset border border-border/60 space-y-1">
                      <span className="text-[10px] font-bold uppercase tracking-wider text-text-secondary">
                        Kunci Tersimpan ({prov.masked_keys.length}):
                      </span>
                      <div className="flex flex-wrap gap-1.5 pt-0.5">
                        {prov.masked_keys.map((k, idx) => (
                          <span
                            key={idx}
                            className="px-2 py-0.5 rounded-md bg-surface font-mono text-xs text-text border border-border/60"
                          >
                            {k}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label
                        htmlFor={`key-input-${prov.id}`}
                        className="text-xs font-semibold text-text-secondary"
                      >
                        {hasExisting ? "Tambah / Ganti API Key" : "Masukkan API Key"}
                      </label>
                      <button
                        type="button"
                        onClick={() =>
                          setShowKeyText((prev) => ({ ...prev, [prov.id]: !prev[prov.id] }))
                        }
                        className="text-[11px] text-primary hover:underline font-semibold"
                      >
                        {showKey ? "Sembunyikan" : "Tampilkan"}
                      </button>
                    </div>
                    <textarea
                      id={`key-input-${prov.id}`}
                      rows={2}
                      value={inputKeys[prov.id] || ""}
                      onChange={(e) =>
                        setInputKeys((prev) => ({ ...prev, [prov.id]: e.target.value }))
                      }
                      placeholder={
                        prov.id === "ollama"
                          ? "Ollama lokal tidak memerlukan key (opsional)"
                          : "sk-... (Bisa masukkan beberapa key dipisahkan baris baru untuk rotasi)"
                      }
                      className={`neu-inset w-full px-3 py-2 text-xs font-mono rounded-xl bg-surface text-text border border-border/60 focus:outline-none resize-none ${
                        !showKey && inputKeys[prov.id] ? "type-password" : ""
                      }`}
                    />
                  </div>

                  <div>
                    <label
                      htmlFor={`base-url-${prov.id}`}
                      className="block text-xs font-semibold text-text-secondary mb-1"
                    >
                      Kustom Base URL (Opsional)
                    </label>
                    <input
                      id={`base-url-${prov.id}`}
                      type="text"
                      value={inputBaseUrls[prov.id] || ""}
                      onChange={(e) =>
                        setInputBaseUrls((prev) => ({ ...prev, [prov.id]: e.target.value }))
                      }
                      placeholder={
                        prov.id === "ollama"
                          ? "http://localhost:11434"
                          : "https://api.openai.com/v1 (kosongkan jika default)"
                      }
                      className="neu-inset w-full px-3 py-2 text-xs font-mono rounded-xl bg-surface text-text border border-border/60 focus:outline-none"
                    />
                  </div>
                </div>

                <div className="flex items-center gap-2 pt-2 border-t border-border/60 flex-wrap">
                  <button
                    type="button"
                    onClick={() => handleSaveKeys(prov.id, false)}
                    disabled={isSaving}
                    className="btn-primary !min-h-[40px] px-4 py-1.5 text-xs font-semibold rounded-xl disabled:opacity-50"
                  >
                    {isSaving ? "Menyimpan..." : "Simpan Kunci"}
                  </button>

                  {hasExisting && (
                    <button
                      type="button"
                      onClick={() => handleSaveKeys(prov.id, true)}
                      disabled={isSaving || prov.is_active}
                      className="btn-soft !min-h-[40px] px-3.5 py-1.5 text-xs font-semibold rounded-xl text-primary disabled:opacity-40"
                    >
                      {prov.is_active ? "Sedang Aktif" : "Jadikan Model Aktif"}
                    </button>
                  )}

                  {hasExisting && (
                    <button
                      type="button"
                      onClick={() => handleDeleteKeys(prov.id)}
                      disabled={isSaving}
                      className="btn-danger !min-h-[40px] px-3 py-1.5 text-xs font-semibold rounded-xl ml-auto disabled:opacity-50"
                      title="Hapus Kunci Provider"
                    >
                      Hapus
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
