"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { ChannelStatus } from "@/types/api";

const CHANNELS_META = [
  { id: "telegram", name: "Telegram Bot", desc: "Two-way bot communication via Telegram Bot API long-polling" },
  { id: "discord", name: "Discord Gateway", desc: "Autonomous agent bot integration with Discord gateway threads" },
  { id: "slack", name: "Slack App", desc: "Interactive bot responses via Slack Events API & Webhooks" },
  { id: "whatsapp", name: "WhatsApp Cloud API", desc: "Meta Graph API webhook adapter for mobile agent messaging" },
];

export default function ChannelsPage() {
  const [channels, setChannels] = useState<Record<string, ChannelStatus>>({});
  const [loading, setLoading] = useState(true);

  const loadChannels = async () => {
    setLoading(true);
    try {
      const data = await api.channels();
      if (Array.isArray(data)) {
        const map: Record<string, ChannelStatus> = {};
        data.forEach((c) => {
          map[c.name.toLowerCase()] = c;
        });
        setChannels(map);
      } else if (data) {
        setChannels(data);
      }
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadChannels();
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Messaging Channels</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Real-time status of external conversational adapters connected to Ryvos
          </p>
        </div>

        <button
          onClick={loadChannels}
          className="btn-soft !min-h-[36px] !py-1 text-xs self-start sm:self-auto"
        >
          <Icons.Refresh className="w-3.5 h-3.5" />
          <span>Refresh</span>
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {CHANNELS_META.map((meta) => {
          const ch = channels[meta.id] || {
            name: meta.name,
            enabled: false,
            status: "not_configured",
          };

          return (
            <div key={meta.id} className="neu-card p-5 flex flex-col justify-between">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Icons.Channels className="w-4 h-4 text-primary" />
                    <span className="font-bold text-sm text-text">{meta.name}</span>
                  </div>
                  <span
                    className={`badge-status text-[10px] ${
                      ch.status === "connected"
                        ? "bg-success/15 text-success"
                        : ch.enabled
                        ? "bg-warning/15 text-warning"
                        : "bg-surface-inset text-text-secondary"
                    }`}
                  >
                    {ch.status}
                  </span>
                </div>
                <p className="text-xs text-text-secondary leading-relaxed mb-4">
                  {meta.desc}
                </p>
              </div>

              <div className="pt-3 border-t border-border flex items-center justify-between text-xs text-text-secondary">
                <span>Enabled: <strong className="text-text">{ch.enabled ? "Yes" : "No"}</strong></span>
                {ch.info && <span className="font-mono text-[10px] truncate max-w-[150px]">{ch.info}</span>}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
