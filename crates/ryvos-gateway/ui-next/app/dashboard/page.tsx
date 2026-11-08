"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { useWebSocket } from "@/lib/context";
import { MetricCard } from "@/components/common/MetricCard";
import { ActivityFeed } from "@/components/common/ActivityFeed";
import { Icons } from "@/components/icons/Icons";
import type { MetricsData } from "@/types/api";

export default function DashboardPage() {
  const { status } = useWebSocket();
  const [metrics, setMetrics] = useState<MetricsData | null>(null);
  const [loading, setLoading] = useState(true);

  const loadMetrics = async () => {
    try {
      const data = await api.metrics();
      setMetrics(data);
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadMetrics();
    const interval = setInterval(loadMetrics, 10000);
    return () => clearInterval(interval);
  }, []);

  const formatBytes = (bytes: number) => {
    if (!bytes) return "0 MB";
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  };

  const formatUptime = (secs: number) => {
    if (!secs) return "0s";
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    if (h > 0) return `${h}j ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Dashboard</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Telemetri dan kesehatan runtime agen.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadMetrics}
            className="btn-soft !min-h-[34px] !py-1 !px-3 text-xs flex items-center gap-1.5"
            title="Segarkan Metrik"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Segarkan</span>
          </button>
          <Link
            href="/chat"
            className="btn-primary !min-h-[34px] !py-1 !px-3 text-xs flex items-center gap-1.5"
          >
            <Icons.Chat className="w-3.5 h-3.5" />
            <span>Buka Chat</span>
          </Link>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          label="Uptime"
          value={formatUptime(metrics?.uptime_secs || 0)}
          sub={`Status: ${status}`}
          icon={<Icons.Dashboard className="w-4 h-4" />}
        />
        <MetricCard
          label="Sesi"
          value={metrics?.total_sessions || 0}
          sub={`${metrics?.active_sessions || 0} aktif`}
          icon={<Icons.Sessions className="w-4 h-4" />}
        />
        <MetricCard
          label="Runs"
          value={metrics?.total_runs || 0}
          sub={`${metrics?.active_runs || 0} berjalan`}
          icon={<Icons.Runs className="w-4 h-4" />}
        />
        <MetricCard
          label="Turn"
          value={metrics?.total_turns || 0}
          sub="Siklus keputusan"
          icon={<Icons.Goals className="w-4 h-4" />}
        />
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          label="RAM (RSS)"
          value={formatBytes(metrics?.rss_bytes || 0)}
          sub="Penggunaan memori"
          icon={<Icons.Config className="w-4 h-4" />}
        />
        <MetricCard
          label="Token Masuk"
          value={(metrics?.total_input_tokens || 0).toLocaleString()}
          sub="Prompt input"
          icon={<Icons.Costs className="w-4 h-4" />}
        />
        <MetricCard
          label="Token Keluar"
          value={(metrics?.total_output_tokens || 0).toLocaleString()}
          sub="Komplesi output"
          icon={<Icons.Costs className="w-4 h-4" />}
        />
        <MetricCard
          label="Total Biaya"
          value={`$${(metrics?.total_cost_usd || 0).toFixed(4)}`}
          sub="Pengeluaran LLM"
          icon={<Icons.Costs className="w-4 h-4" />}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-4">
          <div className="neu-card p-5 border border-border/60">
            <h3 className="text-sm font-bold text-text mb-3">Akses Cepat</h3>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
              <Link
                href="/chat"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Chat className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Chat</div>
                  <div className="text-[10px] text-text-secondary truncate">Prompt langsung</div>
                </div>
              </Link>

              <Link
                href="/runs"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Runs className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Runs</div>
                  <div className="text-[10px] text-text-secondary truncate">Riwayat eksekusi</div>
                </div>
              </Link>

              <Link
                href="/goals"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Goals className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Goals</div>
                  <div className="text-[10px] text-text-secondary truncate">Misi otonom</div>
                </div>
              </Link>

              <Link
                href="/audit"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Audit className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Audit</div>
                  <div className="text-[10px] text-text-secondary truncate">Keamanan & aksi</div>
                </div>
              </Link>

              <Link
                href="/viking"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Viking className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Memory</div>
                  <div className="text-[10px] text-text-secondary truncate">Browser L0/L1/L2</div>
                </div>
              </Link>

              <Link
                href="/settings"
                className="p-3 rounded-xl bg-surface-raised border border-border/60 hover:border-primary transition-all flex items-center gap-3 shadow-raised-sm"
              >
                <div className="p-2 rounded-lg bg-primary-soft text-primary">
                  <Icons.Settings className="w-4 h-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-text truncate">Pengaturan</div>
                  <div className="text-[10px] text-text-secondary truncate">Model & budget</div>
                </div>
              </Link>
            </div>
          </div>
        </div>

        <div>
          <ActivityFeed maxItems={40} />
        </div>
      </div>
    </div>
  );
}
