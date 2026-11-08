"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";

export default function ConfigPage() {
  const [configText, setConfigText] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState("");

  const loadConfig = async () => {
    setLoading(true);
    setStatusMessage("");
    try {
      const data = await api.config.get();
      if (typeof data === "string") {
        setConfigText(data);
      } else {
        setConfigText(JSON.stringify(data, null, 2));
      }
    } catch (err: any) {
      setStatusMessage(`Error loading config: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setStatusMessage("");
    try {
      let parsed: any;
      try {
        parsed = JSON.parse(configText);
      } catch {
        parsed = { raw: configText };
      }
      await api.config.put(parsed);
      setStatusMessage("Configuration saved successfully");
    } catch (err: any) {
      setStatusMessage(`Save failed: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Runtime Configuration</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            View and update the live runtime `config.toml` specification
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadConfig}
            className="btn-soft !min-h-[36px] !py-1 text-xs"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Reload</span>
          </button>
          <button
            onClick={handleSave}
            disabled={saving || loading}
            className="btn-primary !min-h-[36px] !py-1 text-xs disabled:opacity-50"
          >
            <Icons.Check className="w-3.5 h-3.5" />
            <span>{saving ? "Saving..." : "Save Config"}</span>
          </button>
        </div>
      </div>

      {statusMessage && (
        <div className="p-3 rounded-lg text-xs bg-surface-inset border border-border text-primary font-medium">
          {statusMessage}
        </div>
      )}

      <div className="neu-focal p-4">
        <textarea
          rows={24}
          value={configText}
          onChange={(e) => setConfigText(e.target.value)}
          disabled={loading}
          placeholder="Loading config..."
          className="w-full font-mono text-xs leading-relaxed bg-surface-inset p-4 rounded-xl border border-border text-text focus:outline-none focus:border-primary resize-y"
        />
      </div>
    </div>
  );
}
