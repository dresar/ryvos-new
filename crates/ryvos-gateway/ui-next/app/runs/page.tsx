"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { RunRecord } from "@/types/api";

export default function RunsPage() {
  const [runs, setRuns] = useState<RunRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const limit = 20;
  const [loading, setLoading] = useState(true);
  const [selectedRun, setSelectedRun] = useState<RunRecord | null>(null);

  const loadRuns = async () => {
    setLoading(true);
    try {
      const data = await api.runs(limit, page * limit);
      if (data) {
        setRuns(Array.isArray(data.runs) ? data.runs : []);
        setTotal(data.total || (Array.isArray(data.runs) ? data.runs.length : 0));
      }
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadRuns();
  }, [page]);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "running":
        return (
          <span className="badge-status bg-warning/15 text-warning text-[10px]">
            <span className="w-1.5 h-1.5 rounded-full bg-warning animate-pulse" />
            berjalan
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
      default:
        return (
          <span className="badge-status bg-surface-inset text-text-secondary text-[10px]">
            {status}
          </span>
        );
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Runs</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Riwayat eksekusi agen otonom dan metrik token.
          </p>
        </div>

        <button
          onClick={loadRuns}
          className="btn-soft !min-h-[34px] !py-1 !px-3 text-xs flex items-center gap-1.5 self-start sm:self-auto"
          title="Segarkan Riwayat"
        >
          <Icons.Refresh className="w-3.5 h-3.5" />
          <span>Segarkan</span>
        </button>
      </div>

      <div className="table-scroll neu-card border border-border/60">
        <table className="w-full text-left text-xs border-collapse">
          <thead>
            <tr className="border-b border-border/60 bg-surface text-text-secondary">
              <th className="p-3 font-semibold">ID Run</th>
              <th className="p-3 font-semibold">Sesi</th>
              <th className="p-3 font-semibold">Status</th>
              <th className="p-3 font-semibold tabular-nums">Durasi</th>
              <th className="p-3 font-semibold tabular-nums">Turn</th>
              <th className="p-3 font-semibold tabular-nums">Token (In / Out)</th>
              <th className="p-3 font-semibold tabular-nums">Biaya</th>
              <th className="p-3 font-semibold text-right">Aksi</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={8} className="p-8 text-center text-text-secondary">
                  Memuat runs...
                </td>
              </tr>
            ) : runs.length === 0 ? (
              <tr>
                <td colSpan={8} className="p-8 text-center text-text-secondary">
                  Belum ada riwayat eksekusi
                </td>
              </tr>
            ) : (
              (Array.isArray(runs) ? runs : []).map((r) => (
                <tr
                  key={r.run_id}
                  className="border-b border-border/40 hover:bg-surface-inset/40 transition-colors"
                >
                  <td className="p-3 font-mono font-medium text-text">{r.run_id.substring(0, 12)}...</td>
                  <td className="p-3 font-mono text-text-secondary">{r.session_id ? r.session_id.substring(0, 10) : "-"}</td>
                  <td className="p-3">{getStatusBadge(r.status)}</td>
                  <td className="p-3 font-mono tabular-nums text-text-secondary">
                    {r.duration_ms ? `${(r.duration_ms / 1000).toFixed(1)}s` : "-"}
                  </td>
                  <td className="p-3 font-mono tabular-nums text-text">{r.turns || 0}</td>
                  <td className="p-3 font-mono tabular-nums text-text-secondary">
                    {r.input_tokens || 0} / {r.output_tokens || 0}
                  </td>
                  <td className="p-3 font-mono tabular-nums text-text">
                    ${(r.cost_usd || 0).toFixed(4)}
                  </td>
                  <td className="p-3 text-right">
                    <button
                      onClick={() => setSelectedRun(r)}
                      className="text-primary hover:underline font-semibold"
                    >
                      Detail
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <div className="flex items-center justify-between text-xs text-text-secondary">
        <div>Halaman {page + 1}</div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setPage((p) => Math.max(0, p - 1))}
            disabled={page === 0}
            className="btn-soft !min-h-[30px] !py-1 !px-3 text-xs disabled:opacity-40"
          >
            Sebelumnya
          </button>
          <button
            onClick={() => setPage((p) => p + 1)}
            disabled={runs.length < limit}
            className="btn-soft !min-h-[30px] !py-1 !px-3 text-xs disabled:opacity-40"
          >
            Selanjutnya
          </button>
        </div>
      </div>

      {selectedRun && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-sm p-4">
          <div className="neu-focal p-5 w-full max-w-lg relative max-h-[85vh] overflow-y-auto border border-border/60">
            <div className="flex items-center justify-between pb-3 border-b border-border/60 mb-4">
              <h3 className="text-sm font-bold text-text">Detail Run</h3>
              <button
                onClick={() => setSelectedRun(null)}
                className="p-1 text-text-secondary hover:text-text rounded-md hover:bg-surface-inset"
              >
                <Icons.X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <span className="text-text-secondary">ID Run: </span>
                <span className="font-mono font-bold text-text">{selectedRun.run_id}</span>
              </div>
              <div>
                <span className="text-text-secondary">ID Sesi: </span>
                <span className="font-mono text-text">{selectedRun.session_id}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-text-secondary">Status: </span>
                {getStatusBadge(selectedRun.status)}
              </div>
              <div>
                <span className="text-text-secondary">Waktu Mulai: </span>
                <span className="text-text">{new Date(selectedRun.started_at).toLocaleString()}</span>
              </div>
              {selectedRun.ended_at && (
                <div>
                  <span className="text-text-secondary">Waktu Selesai: </span>
                  <span className="text-text">{new Date(selectedRun.ended_at).toLocaleString()}</span>
                </div>
              )}
              <div className="grid grid-cols-2 gap-3 p-3 rounded-xl bg-surface-inset border border-border/60">
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Turn</div>
                  <div className="font-mono font-bold text-base text-text">{selectedRun.turns}</div>
                </div>
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Biaya</div>
                  <div className="font-mono font-bold text-base text-text">${(selectedRun.cost_usd || 0).toFixed(4)}</div>
                </div>
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Token Masuk</div>
                  <div className="font-mono text-text">{selectedRun.input_tokens}</div>
                </div>
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Token Keluar</div>
                  <div className="font-mono text-text">{selectedRun.output_tokens}</div>
                </div>
              </div>

              {selectedRun.error && (
                <div className="p-3 rounded-xl border border-danger/30 bg-danger/5 text-danger font-mono text-[11px] whitespace-pre-wrap">
                  {selectedRun.error}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
