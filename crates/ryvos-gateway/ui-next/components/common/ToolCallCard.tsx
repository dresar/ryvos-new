"use client";

import React, { useState } from "react";
import { Icons } from "@/components/icons/Icons";
import type { ToolCallItem } from "@/types/api";

export function ToolCallCard({ tool }: { tool: ToolCallItem }) {
  const [open, setOpen] = useState(false);

  const getStatusBadge = () => {
    switch (tool.status) {
      case "running":
        return (
          <span className="badge-status bg-warning/15 text-warning text-[10px]">
            <span className="w-1.5 h-1.5 rounded-full bg-warning animate-pulse" />
            proses
          </span>
        );
      case "completed":
        return (
          <span className="badge-status bg-success/15 text-success text-[10px]">
            <span className="w-1.5 h-1.5 rounded-full bg-success" />
            selesai
          </span>
        );
      case "failed":
        return (
          <span className="badge-status bg-danger/15 text-danger text-[10px]">
            <span className="w-1.5 h-1.5 rounded-full bg-danger" />
            gagal
          </span>
        );
    }
  };

  return (
    <div className="mb-2 rounded-xl border border-border/60 bg-surface-inset overflow-hidden text-xs">
      <button
        onClick={() => setOpen(!open)}
        className="w-full px-3 py-2 flex items-center justify-between text-left hover:bg-surface/60 transition-colors"
      >
        <div className="flex items-center gap-2 min-w-0">
          <Icons.Wrench className="w-3.5 h-3.5 text-primary shrink-0" />
          <span className="font-mono font-semibold text-text truncate">{tool.name}</span>
          {tool.elapsed_ms !== undefined && (
            <span className="text-[10px] text-text-secondary font-mono">
              {tool.elapsed_ms}ms
            </span>
          )}
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {getStatusBadge()}
          <Icons.ChevronDown
            className={`w-3.5 h-3.5 text-text-secondary transition-transform ${open ? "rotate-180" : ""}`}
          />
        </div>
      </button>

      {open && (
        <div className="p-3 border-t border-border/60 space-y-2 bg-surface-inset/60 font-mono text-[11px]">
          <div>
            <div className="text-[10px] uppercase font-bold text-text-secondary mb-1">Input</div>
            <pre className="p-2 rounded-lg bg-surface border border-border/60 text-text whitespace-pre-wrap max-h-48 overflow-y-auto">
              {tool.input}
            </pre>
          </div>
          {tool.output && (
            <div>
              <div className="text-[10px] uppercase font-bold text-text-secondary mb-1">Output</div>
              <pre className="p-2 rounded-lg bg-surface border border-border/60 text-text whitespace-pre-wrap max-h-48 overflow-y-auto">
                {tool.output}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
