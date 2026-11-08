"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { useWebSocket } from "@/lib/context";
import { Icons } from "@/components/icons/Icons";
import { PROVIDERS_CONFIG } from "@/components/common/ModelSelectorModal";

export default function SettingsPage() {
  const { modelInfo: wsModelInfo, status } = useWebSocket();
  const [provider, setProvider] = useState("gemini");
  const [modelId, setModelId] = useState("gemini-2.5-flash");
  const [apiKeysInput, setApiKeysInput] = useState("");
  const [dailyBudget, setDailyBudget] = useState(10.0);
  const [warningThreshold, setWarningThreshold] = useState(80);
  const [savingBudget, setSavingBudget] = useState(false);
  const [savingModel, setSavingModel] = useState(false);
  const [message, setMessage] = useState("");
  const [healthData, setHealthData] = useState<any>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const [b, m, h] = await Promise.allSettled([
        api.budget.get(),
        api.model.get(),
        api.health(),
      ]);
      if (b.status === "fulfilled" && b.value) {
        if (b.value.daily_limit_usd) setDailyBudget(b.value.daily_limit_usd);
        if (b.value.warning_threshold_pct) setWarningThreshold(b.value.warning_threshold_pct);
      }
      if (m.status === "fulfilled" && m.value) {
        setProvider(m.value.provider || "gemini");
        setModelId(m.value.model_id || "gemini-2.5-flash");
        if (m.value.api_keys) setApiKeysInput(m.value.api_keys.join("\n"));
      }
      if (h.status === "fulfilled") setHealthData(h.value);
    } catch {}
  };

  const handleSaveBudget = async (e: React.FormEvent) => {
    e.preventDefault();
    setSavingBudget(true);
    setMessage("");
    try {
      await api.budget.put({
        daily_limit_usd: Number(dailyBudget),
        warning_threshold_pct: Number(warningThreshold),
      });
      setMessage("Batas anggaran berhasil diperbarui.");
    } catch (err: any) {
      setMessage(`Gagal: ${err.message}`);
    } finally {
      setSavingBudget(false);
    }
  };

  const handleSaveModel = async (e: React.FormEvent) => {
    e.preventDefault();
    setSavingModel(true);
    setMessage("");
    try {
      const keys = apiKeysInput
        .split("\n")
        .map((k) => k.trim())
        .filter(Boolean);
      await api.model.put({
        provider,
        model_id: modelId,
        api_keys: keys,
      });
      setMessage("Konfigurasi model & key rotator diperbarui.");
    } catch (err: any) {
      setMessage(`Gagal: ${err.message}`);
    } finally {
      setSavingModel(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-text">Pengaturan</h2>
        <p className="text-xs text-text-secondary mt-0.5">
          Konfigurasi model, kuota anggaran, dan kunci API.
        </p>
      </div>

      {message && (
        <div className="p-3 rounded-xl text-xs bg-surface-inset border border-border/60 text-primary font-medium">
          {message}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="neu-card p-5 border border-border/60">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-bold text-text">Model Default & Key Rotator</h3>
            <Link
              href="/keys"
              className="text-xs text-primary font-semibold hover:underline inline-flex items-center gap-1"
            >
              <span>Manajemen API Key</span>
              <span>&rarr;</span>
            </Link>
          </div>
          <form onSubmit={handleSaveModel} className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-text-secondary mb-1">
                Provider
              </label>
              <select
                value={provider}
                onChange={(e) => {
                  const p = e.target.value;
                  setProvider(p);
                  const found = PROVIDERS_CONFIG.find((x) => x.id === p);
                  if (found && found.presets.length > 0) setModelId(found.presets[0].id);
                }}
                className="neu-inset w-full px-3 py-2 text-xs focus:outline-none bg-surface-inset border border-border/60 rounded-lg"
              >
                {PROVIDERS_CONFIG.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-text-secondary mb-1">
                Model ID
              </label>
              <input
                type="text"
                value={modelId}
                onChange={(e) => setModelId(e.target.value)}
                className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none border border-border/60 rounded-lg"
                required
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-text-secondary mb-1">
                Key Pool (Satu kunci per baris)
              </label>
              <textarea
                rows={4}
                value={apiKeysInput}
                onChange={(e) => setApiKeysInput(e.target.value)}
                placeholder="sk-...\nsk-..."
                className="neu-inset w-full p-2.5 text-xs font-mono focus:outline-none border border-border/60 rounded-lg resize-none"
              />
              <p className="text-[10px] text-text-secondary mt-1">
                Kunci diputar otomatis untuk menghindari rate limit provider.
              </p>
            </div>

            <button
              type="submit"
              disabled={savingModel}
              className="btn-primary !min-h-[36px] !py-1 !px-4 text-xs disabled:opacity-50"
            >
              {savingModel ? "Menyimpan..." : "Simpan Model"}
            </button>
          </form>
        </div>

        <div className="space-y-6">
          <div className="neu-card p-5 border border-border/60">
            <h3 className="text-sm font-bold text-text mb-3">Batas Anggaran</h3>
            <form onSubmit={handleSaveBudget} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">
                  Batas Harian (USD)
                </label>
                <input
                  type="number"
                  step="0.5"
                  value={dailyBudget}
                  onChange={(e) => setDailyBudget(Number(e.target.value))}
                  className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none border border-border/60 rounded-lg"
                  required
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">
                  Ambang Peringatan (%)
                </label>
                <input
                  type="number"
                  min={10}
                  max={100}
                  value={warningThreshold}
                  onChange={(e) => setWarningThreshold(Number(e.target.value))}
                  className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none border border-border/60 rounded-lg"
                  required
                />
              </div>

              <button
                type="submit"
                disabled={savingBudget}
                className="btn-primary !min-h-[36px] !py-1 !px-4 text-xs disabled:opacity-50"
              >
                {savingBudget ? "Menyimpan..." : "Simpan Anggaran"}
              </button>
            </form>
          </div>

          <div className="neu-card p-5 border border-border/60">
            <h3 className="text-sm font-bold text-text mb-3">Diagnostik Sistem</h3>
            <div className="space-y-2 text-xs font-mono">
              <div className="flex justify-between py-1 border-b border-border/40">
                <span className="text-text-secondary">Versi Biner:</span>
                <span className="text-text font-bold">v0.9.0</span>
              </div>
              <div className="flex justify-between py-1 border-b border-border/40">
                <span className="text-text-secondary">Runtime Rust:</span>
                <span className="text-text">Axum + Tokio</span>
              </div>
              <div className="flex justify-between py-1 border-b border-border/40">
                <span className="text-text-secondary">Batas Memori:</span>
                <span className="text-text">4 GB</span>
              </div>
              <div className="flex justify-between py-1 border-b border-border/40">
                <span className="text-text-secondary">Embed UI:</span>
                <span className="text-text">Static Export</span>
              </div>
              <div className="flex justify-between py-1">
                <span className="text-text-secondary">Status Gateway:</span>
                <span className="text-success font-bold capitalize">{status}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
