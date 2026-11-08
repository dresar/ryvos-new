"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { MetricCard } from "@/components/common/MetricCard";
import { Icons } from "@/components/icons/Icons";
import type { CostRecord } from "@/types/api";

export default function CostsPage() {
  const [costs, setCosts] = useState<CostRecord[]>([]);
  const [totalUsd, setTotalUsd] = useState(0);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState("7d");
  const [groupBy, setGroupBy] = useState("model");

  const loadCosts = async () => {
    setLoading(true);
    try {
      const data = await api.costs();
      if (data) {
        setCosts(Array.isArray(data.costs) ? data.costs : []);
        setTotalUsd(data.total_usd || 0);
      }
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCosts();
  }, [timeRange, groupBy]);

  const totalInputTokens = (Array.isArray(costs) ? costs : []).reduce((acc, c) => acc + (c.input_tokens || 0), 0);
  const totalOutputTokens = (Array.isArray(costs) ? costs : []).reduce((acc, c) => acc + (c.output_tokens || 0), 0);

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">LLM Token & Cost Telemetry</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Aggregated expenditure across multi-provider LLM models and agent runs
          </p>
        </div>

        <button
          onClick={loadCosts}
          className="btn-soft !min-h-[36px] !py-1 text-xs self-start sm:self-auto"
        >
          <Icons.Refresh className="w-3.5 h-3.5" />
          <span>Refresh</span>
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <MetricCard
          label="Total Expenditure"
          value={`$${totalUsd.toFixed(4)}`}
          sub="Cumulative API cost"
          icon={<Icons.Costs className="w-4 h-4" />}
        />
        <MetricCard
          label="Total Input Tokens"
          value={totalInputTokens.toLocaleString()}
          sub="Prompt processing"
          icon={<Icons.Runs className="w-4 h-4" />}
        />
        <MetricCard
          label="Total Output Tokens"
          value={totalOutputTokens.toLocaleString()}
          sub="Completion generation"
          icon={<Icons.Goals className="w-4 h-4" />}
        />
      </div>

      <div className="table-scroll">
        <table className="w-full text-left text-xs border-collapse">
          <thead>
            <tr className="border-b border-border bg-surface text-text-secondary">
              <th className="p-3 font-semibold">Timestamp</th>
              <th className="p-3 font-semibold">Provider</th>
              <th className="p-3 font-semibold">Model</th>
              <th className="p-3 font-semibold">Session</th>
              <th className="p-3 font-semibold tabular-nums">Input Tokens</th>
              <th className="p-3 font-semibold tabular-nums">Output Tokens</th>
              <th className="p-3 font-semibold tabular-nums text-right">Cost (USD)</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={7} className="p-8 text-center text-text-secondary">
                  Loading cost records...
                </td>
              </tr>
            ) : costs.length === 0 ? (
              <tr>
                <td colSpan={7} className="p-8 text-center text-text-secondary">
                  No cost records available
                </td>
              </tr>
            ) : (
              (Array.isArray(costs) ? costs : []).map((c, i) => (
                <tr
                  key={c.id || i}
                  className="border-b border-border/50 hover:bg-surface-inset/50 transition-colors"
                >
                  <td className="p-3 font-mono text-text-secondary whitespace-nowrap">
                    {new Date(c.timestamp).toLocaleDateString()} {new Date(c.timestamp).toLocaleTimeString()}
                  </td>
                  <td className="p-3 font-semibold uppercase text-[10px] text-primary">{c.provider}</td>
                  <td className="p-3 font-mono text-text">{c.model}</td>
                  <td className="p-3 font-mono text-text-secondary">{c.session_id ? c.session_id.substring(0, 10) : "-"}</td>
                  <td className="p-3 font-mono tabular-nums text-text-secondary">{c.input_tokens}</td>
                  <td className="p-3 font-mono tabular-nums text-text-secondary">{c.output_tokens}</td>
                  <td className="p-3 font-mono tabular-nums text-right font-bold text-text">
                    ${(c.cost_usd || 0).toFixed(4)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
