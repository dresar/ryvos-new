"use client";

import React, { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { Icons } from "@/components/icons/Icons";

const COMMAND_ITEMS = [
  { path: "/chat", title: "Chat", desc: "Interact with autonomous agent", icon: Icons.Chat },
  { path: "/dashboard", title: "Dashboard", desc: "Cluster overview & live telemetry", icon: Icons.Dashboard },
  { path: "/sessions", title: "Sessions", desc: "Manage conversation sessions", icon: Icons.Sessions },
  { path: "/runs", title: "Runs", desc: "Execution traces, status & tokens", icon: Icons.Runs },
  { path: "/goals", title: "Goals", desc: "Autonomous goal runner & judge evaluation", icon: Icons.Goals },
  { path: "/audit", title: "Audit Trail", desc: "Security events, SSRF blocks & decisions", icon: Icons.Audit },
  { path: "/costs", title: "Costs", desc: "LLM token usage & API expenditure", icon: Icons.Costs },
  { path: "/integrations", title: "Integrations", desc: "Model Context Protocol & tool connectors", icon: Icons.Integrations },
  { path: "/channels", title: "Channels", desc: "Telegram, Discord, Slack & WhatsApp", icon: Icons.Channels },
  { path: "/viking", title: "Viking Browser", desc: "Hierarchical memory viewer (L0/L1/L2)", icon: Icons.Viking },
  { path: "/cron", title: "Graph & Cron", desc: "Autonomous scheduled jobs & heartbeats", icon: Icons.Graph },
  { path: "/keys", title: "API Keys", desc: "Manajemen Kunci & Rotasi Provider AI", icon: Icons.Key },
  { path: "/config", title: "Config Editor", desc: "Live runtime config.toml editor", icon: Icons.Config },
  { path: "/settings", title: "Settings", desc: "Cluster limits, key rotator & diagnostics", icon: Icons.Settings },
];

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const router = useRouter();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
      if (e.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const filtered = COMMAND_ITEMS.filter(
    (item) =>
      item.title.toLowerCase().includes(query.toLowerCase()) ||
      item.desc.toLowerCase().includes(query.toLowerCase()) ||
      item.path.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleSelect = (path: string) => {
    router.push(path);
    setOpen(false);
    setQuery("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % (filtered.length || 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + filtered.length) % (filtered.length || 1));
    } else if (e.key === "Enter" && filtered[selectedIndex]) {
      e.preventDefault();
      handleSelect(filtered[selectedIndex].path);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-bg/80 backdrop-blur-sm p-4">
      <div
        className="fixed inset-0"
        onClick={() => setOpen(false)}
      />
      <div className="neu-focal w-full max-w-lg relative z-10 overflow-hidden shadow-floating">
        <div className="flex items-center gap-3 px-4 py-3 border-b border-border bg-surface">
          <Icons.Search className="w-5 h-5 text-text-secondary" />
          <input
            type="text"
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or navigate (e.g. Chat, Costs)..."
            className="w-full bg-transparent text-sm text-text placeholder:text-text-secondary/50 focus:outline-none"
          />
          <kbd className="px-2 py-0.5 rounded text-[10px] bg-surface-inset border border-border text-text-secondary">
            ESC
          </kbd>
        </div>

        <div className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-xs text-text-secondary">
              No results found for &quot;{query}&quot;
            </div>
          ) : (
            filtered.map((item, idx) => {
              const IconComp = item.icon;
              const isSelected = idx === selectedIndex;
              return (
                <button
                  key={item.path}
                  onClick={() => handleSelect(item.path)}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`w-full flex items-center justify-between p-2.5 rounded-md text-left transition-colors ${
                    isSelected
                      ? "bg-primary-soft text-primary font-semibold"
                      : "text-text hover:bg-surface-inset"
                  }`}
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <IconComp className={`w-4 h-4 shrink-0 ${isSelected ? "text-primary" : "text-text-secondary"}`} />
                    <div className="truncate">
                      <div className="text-xs">{item.title}</div>
                      <div className="text-[11px] text-text-secondary truncate">{item.desc}</div>
                    </div>
                  </div>
                  <span className="text-[10px] font-mono text-text-secondary shrink-0 ml-2">
                    {item.path}
                  </span>
                </button>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
