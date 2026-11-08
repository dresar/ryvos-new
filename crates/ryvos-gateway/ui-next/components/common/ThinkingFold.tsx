"use client";

import React, { useState } from "react";
import { Icons } from "@/components/icons/Icons";

export function ThinkingFold({ think }: { think: string }) {
  const [open, setOpen] = useState(false);

  if (!think.trim()) return null;

  return (
    <div className="mb-3 rounded-xl border border-border/60 bg-surface-inset overflow-hidden transition-all text-xs">
      <button
        onClick={() => setOpen(!open)}
        className="w-full px-3 py-2 flex items-center justify-between text-text-secondary hover:text-text bg-surface/50 font-medium transition-colors"
      >
        <div className="flex items-center gap-2">
          <Icons.Brain className="w-3.5 h-3.5 text-primary" />
          <span>Proses Berpikir</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-primary/10 text-primary">
            {open ? "tutup" : "buka"}
          </span>
        </div>
        <Icons.ChevronDown
          className={`w-3.5 h-3.5 transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>

      {open && (
        <div className="p-3 text-text-secondary whitespace-pre-wrap font-mono text-[11px] leading-relaxed border-t border-border/60 bg-surface-inset/80 max-h-60 overflow-y-auto">
          {think}
        </div>
      )}
    </div>
  );
}
