"use client";

import React from "react";
import { useWebSocket } from "@/lib/context";
import type { ActivityEvent } from "@/types/api";

export function ActivityFeed({ maxItems = 50 }: { maxItems?: number }) {
  const { activityFeed } = useWebSocket();
  const displayed = activityFeed.slice(0, maxItems);

  const getDotColor = (kind: string) => {
    if (kind.includes("error") || kind.includes("alert") || kind.includes("blocked") || kind.includes("exceeded")) {
      return "bg-danger";
    }
    if (kind.includes("warning") || kind.includes("stall") || kind.includes("approval")) {
      return "bg-warning";
    }
    if (kind.includes("complete") || kind.includes("ok")) {
      return "bg-success";
    }
    return "bg-primary";
  };

  return (
    <div className="neu-card p-4 flex flex-col h-full border border-border/60">
      <div className="flex items-center justify-between pb-3 border-b border-border/60">
        <h3 className="text-sm font-bold text-text">Aktivitas Runtime</h3>
        <span className="text-xs text-text-secondary tabular-nums">
          {displayed.length} event
        </span>
      </div>

      <div className="flex-1 overflow-y-auto mt-3 space-y-2 pr-1 no-scrollbar max-h-[400px]">
        {displayed.length === 0 ? (
          <div className="text-center py-8 text-xs text-text-secondary">
            Menunggu event runtime...
          </div>
        ) : (
          displayed.map((item, idx) => (
            <div
              key={idx}
              className="text-xs p-2.5 rounded-lg bg-surface-inset border border-border/60 flex items-start gap-2.5"
            >
              <div className={`w-2 h-2 rounded-full mt-1 shrink-0 ${getDotColor(item.kind)}`} />
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-1">
                  <span className="font-semibold text-text truncate">{item.kind}</span>
                  <span className="text-[10px] text-text-secondary shrink-0 tabular-nums">
                    {item.time}
                  </span>
                </div>
                <div className="text-text-secondary truncate mt-0.5">{item.detail}</div>
                {item.session && (
                  <div className="text-[10px] text-text-secondary mt-1 font-mono">
                    sesi: {item.session}
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
