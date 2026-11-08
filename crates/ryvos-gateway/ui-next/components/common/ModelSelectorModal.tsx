"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
import { Icons } from "@/components/icons/Icons";
import { api } from "@/lib/api";
import type { ProviderKeyInfo } from "@/types/api";

export interface PresetModel {
  id: string;
  name: string;
}

export interface ProviderConfig {
  id: string;
  name: string;
  presets: PresetModel[];
}

export const PROVIDERS_CONFIG: ProviderConfig[] = [
  {
    id: "gemini",
    name: "Gemini",
    presets: [
      { id: "gemini-3.6-flash", name: "gemini-3.6-flash" },
      { id: "gemini-2.5-flash", name: "gemini-2.5-flash" },
      { id: "gemini-2.5-pro", name: "gemini-2.5-pro" },
    ],
  },
  {
    id: "anthropic",
    name: "Claude",
    presets: [
      { id: "claude-3-7-sonnet", name: "claude-3-7-sonnet" },
      { id: "claude-sonnet-4-20250514", name: "claude-sonnet-4" },
      { id: "claude-3-5-sonnet-20241022", name: "claude-3-5-sonnet" },
    ],
  },
  {
    id: "openai",
    name: "OpenAI",
    presets: [
      { id: "gpt-4.5-preview", name: "gpt-4.5-preview" },
      { id: "gpt-4o", name: "gpt-4o" },
      { id: "gpt-4o-mini", name: "gpt-4o-mini" },
    ],
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    presets: [
      { id: "deepseek-chat", name: "deepseek-chat" },
      { id: "deepseek-reasoner", name: "deepseek-reasoner" },
    ],
  },
  {
    id: "groq",
    name: "Groq",
    presets: [
      { id: "llama-3.3-70b-versatile", name: "llama-3.3-70b-versatile" },
      { id: "llama-3.1-8b-instant", name: "llama-3.1-8b-instant" },
    ],
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    presets: [
      { id: "google/gemini-2.5-flash", name: "google/gemini-2.5-flash" },
      { id: "anthropic/claude-3.7-sonnet", name: "anthropic/claude-3.7-sonnet" },
    ],
  },
  {
    id: "ollama",
    name: "Ollama (Local)",
    presets: [
      { id: "llama3.2", name: "llama3.2" },
      { id: "qwen2.5:coder", name: "qwen2.5:coder" },
    ],
  },
  {
    id: "custom",
    name: "Kustom",
    presets: [],
  },
];

interface ModelSelectorModalProps {
  isOpen: boolean;
  onClose: () => void;
  currentProvider: string;
  currentModel: string;
  currentKeys?: string[];
  onApply: (provider: string, modelId: string, keys: string[]) => Promise<void>;
}

export function ModelSelectorModal({
  isOpen,
  onClose,
  currentProvider,
  currentModel,
  currentKeys = [],
  onApply,
}: ModelSelectorModalProps) {
  const [providers, setProviders] = useState<ProviderKeyInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeProviderId, setActiveProviderId] = useState(currentProvider || "");
  const [selectedModelId, setSelectedModelId] = useState(currentModel || "");
  const [customModelId, setCustomModelId] = useState("");
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [saving, setSaving] = useState(false);
  const [errorMsg, setErrorMsg] = useState("");

  const loadProviderKeys = async () => {
    setLoading(true);
    setErrorMsg("");
    try {
      const res = await api.keys.list();
      const all = res.providers || [];
      const active = all.filter((p) => p.has_key);
      setProviders(active);

      const isCurrentActive = active.some((p) => p.id === currentProvider);
      if (isCurrentActive) {
        setActiveProviderId(currentProvider);
        setSelectedModelId(currentModel);
      } else if (active.length > 0) {
        setActiveProviderId(active[0].id);
        setSelectedModelId(active[0].default_model || active[0].models[0]?.id || "");
      } else {
        setActiveProviderId("");
        setSelectedModelId("");
      }
    } catch (err: any) {
      setErrorMsg(err?.message || "Gagal memuat provider ber-API Key");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      setApiKeyInput("");
      setCustomModelId("");
      loadProviderKeys();
    }
  }, [isOpen, currentProvider, currentModel]);

  if (!isOpen) return null;

  const activeProvider = providers.find((p) => p.id === activeProviderId);

  const handleProviderSelect = (provId: string) => {
    setActiveProviderId(provId);
    const target = providers.find((p) => p.id === provId);
    if (target && target.models.length > 0) {
      setSelectedModelId(target.default_model || target.models[0].id);
      setCustomModelId("");
    } else {
      setSelectedModelId("");
    }
  };

  const handlePresetSelect = (presetId: string) => {
    setSelectedModelId(presetId);
    setCustomModelId("");
  };

  const finalModelId = customModelId.trim() || selectedModelId;

  const handleSave = async () => {
    if (!finalModelId) return;
    setSaving(true);
    setErrorMsg("");
    try {
      let keysToSend = currentKeys;
      if (apiKeyInput.trim()) {
        const parsed = apiKeyInput
          .split(/[\n,;]/)
          .map((k) => k.trim())
          .filter((k) => k.length > 0);
        keysToSend = parsed;

        await api.keys.put({
          provider: activeProviderId,
          api_keys: parsed,
          default_model: finalModelId,
          activate: true,
        });
      }

      await onApply(activeProviderId, finalModelId, keysToSend);
      onClose();
    } catch (err: any) {
      setErrorMsg(err?.message || "Gagal menerapkan model");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-bg/80 backdrop-blur-sm animate-fade-in">
      <div className="neu-focal w-full max-w-sm sm:max-w-md p-4 sm:p-5 flex flex-col gap-3.5 rounded-2xl border border-border/60 shadow-floating bg-surface">
        <div className="flex items-center justify-between pb-2 border-b border-border/60">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-primary-soft text-primary flex items-center justify-center shadow-raised-sm">
              <Icons.Cpu className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-text">Pilih Model Aktif</h3>
              <span className="text-[10px] text-text-secondary block">
                Hanya model dengan API Key aktif yang ditampilkan
              </span>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-text-secondary hover:text-text hover:bg-surface-inset transition-colors"
            title="Tutup"
          >
            <Icons.X className="w-4 h-4" />
          </button>
        </div>

        {loading ? (
          <div className="p-8 text-center text-xs text-text-secondary animate-pulse">
            Memeriksa ketersediaan API Key...
          </div>
        ) : providers.length === 0 ? (
          <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-center space-y-3">
            <div className="w-10 h-10 rounded-full bg-red-500/20 text-danger flex items-center justify-center mx-auto">
              <Icons.Key className="w-5 h-5" />
            </div>
            <div>
              <h4 className="text-xs font-bold text-danger">Tidak Ada Model Ber-API Key</h4>
              <p className="text-[11px] text-text-secondary mt-1 leading-relaxed">
                Dilarang keras memilih model tanpa API Key. Belum ada API Key yang dikonfigurasi di sistem.
              </p>
            </div>
            <div className="pt-1">
              <Link
                href="/keys"
                onClick={onClose}
                className="btn-primary inline-flex items-center justify-center gap-1.5 w-full !min-h-[40px] px-4 py-2 rounded-xl text-xs font-semibold shadow-raised-sm"
              >
                <Icons.Key className="w-4 h-4" />
                <span>Buka Manajemen API Key</span>
              </Link>
            </div>
          </div>
        ) : (
          <>
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <span className="text-xs font-semibold text-text-secondary">
                  Provider Teraktivasi ({providers.length})
                </span>
                <Link
                  href="/keys"
                  onClick={onClose}
                  className="text-[11px] text-primary hover:underline font-semibold"
                >
                  Kelola API Key &rarr;
                </Link>
              </div>

              <div className="flex flex-wrap gap-1.5 p-1 rounded-xl bg-surface-inset border border-border/60">
                {providers.map((p) => {
                  const active = p.id === activeProviderId;
                  return (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => handleProviderSelect(p.id)}
                      className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all flex items-center gap-1.5 ${
                        active
                          ? "bg-primary text-white shadow-raised-sm"
                          : "text-text-secondary hover:text-text hover:bg-surface-raised/60"
                      }`}
                    >
                      <span>{p.name}</span>
                      <span className="text-[10px] opacity-80 tabular-nums">({p.key_count})</span>
                    </button>
                  );
                })}
              </div>
            </div>

            {activeProvider && activeProvider.models.length > 0 && (
              <div className="space-y-1.5">
                <span className="text-xs font-semibold text-text-secondary block">
                  Model Tersedia untuk {activeProvider.name}:
                </span>
                <div className="flex flex-wrap gap-1.5">
                  {activeProvider.models.map((preset) => {
                    const isSelected = selectedModelId === preset.id && !customModelId.trim();
                    return (
                      <button
                        key={preset.id}
                        type="button"
                        onClick={() => handlePresetSelect(preset.id)}
                        className={`px-2.5 py-1.5 rounded-lg text-xs font-mono transition-all border ${
                          isSelected
                            ? "bg-primary-soft border-primary text-primary font-bold shadow-raised-sm"
                            : "bg-surface-raised border-border/60 text-text hover:border-primary/40"
                        }`}
                      >
                        {preset.name}
                      </button>
                    );
                  })}
                </div>
              </div>
            )}

            {activeProvider && activeProvider.masked_keys.length > 0 && (
              <div className="px-2.5 py-1.5 rounded-xl bg-surface-inset border border-border/60 flex items-center justify-between text-xs font-mono">
                <span className="text-[11px] text-text-secondary">Key Aktif:</span>
                <span className="text-text font-bold truncate max-w-[200px]">
                  {activeProvider.masked_keys[0]}
                </span>
              </div>
            )}

            <div>
              <input
                type="text"
                value={customModelId}
                onChange={(e) => setCustomModelId(e.target.value)}
                placeholder="ID Model Spesifik (opsional jika memilih preset di atas)..."
                className="neu-inset w-full px-3 py-2 text-xs font-mono placeholder:text-text-secondary/50 focus:outline-none border border-border/60 rounded-xl"
              />
            </div>

            <div>
              <input
                type="password"
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                placeholder="Perbarui / Tambah Key untuk provider ini (opsional)..."
                className="neu-inset w-full px-3 py-2 text-xs font-mono placeholder:text-text-secondary/50 focus:outline-none border border-border/60 rounded-xl"
              />
            </div>

            {errorMsg && (
              <div className="p-2 rounded-lg bg-red-500/10 text-[11px] text-danger">
                {errorMsg}
              </div>
            )}

            <div className="flex items-center gap-2 pt-1">
              <button
                type="button"
                onClick={onClose}
                className="btn-outline flex-1 !min-h-[40px] !py-1.5 text-xs rounded-xl"
              >
                Batal
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={saving || !finalModelId}
                className="btn-primary flex-1 !min-h-[40px] !py-1.5 text-xs font-semibold rounded-xl shadow-raised-sm disabled:opacity-50"
              >
                {saving ? "Menyimpan..." : "Terapkan Model"}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
