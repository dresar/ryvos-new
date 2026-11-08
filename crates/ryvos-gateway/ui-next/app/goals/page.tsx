"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { GoalRun } from "@/types/api";

export default function GoalsPage() {
  const [goals, setGoals] = useState<GoalRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [prompt, setPrompt] = useState("");
  const [maxTurns, setMaxTurns] = useState(15);
  const [judgeModel, setJudgeModel] = useState("gemini-2.5-flash");
  const [running, setRunning] = useState(false);
  const [selectedGoal, setSelectedGoal] = useState<GoalRun | null>(null);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await api.goals.history();
      setGoals(Array.isArray(data) ? data : []);
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  const handleRunGoal = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || running) return;

    setRunning(true);
    try {
      await api.goals.run({
        prompt: prompt.trim(),
        max_turns: maxTurns,
        judge_model: judgeModel,
      });
      setPrompt("");
      loadHistory();
    } catch (err: any) {
      alert(`Gagal: ${err.message}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold text-text">Goals</h2>
        <p className="text-xs text-text-secondary mt-0.5">
          Misi otonom dengan evaluasi juri independen.
        </p>
      </div>

      <div className="neu-card p-5 border border-border/60">
        <h3 className="text-sm font-bold text-text mb-3">Misi Baru</h3>
        <form onSubmit={handleRunGoal} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-text-secondary mb-1">
              Instruksi Misi
            </label>
            <textarea
              rows={3}
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Tuliskan tujuan misi..."
              className="neu-inset w-full p-3 text-xs placeholder:text-text-secondary/50 focus:outline-none border border-border/60 resize-none"
            />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <div className="flex items-center justify-between text-xs font-semibold text-text-secondary mb-1">
                <span>Batas Turn</span>
                <span className="font-mono text-primary font-bold">{maxTurns}</span>
              </div>
              <input
                type="range"
                min={3}
                max={50}
                value={maxTurns}
                onChange={(e) => setMaxTurns(Number(e.target.value))}
                className="w-full accent-primary"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-text-secondary mb-1">
                Model Juri
              </label>
              <input
                type="text"
                value={judgeModel}
                onChange={(e) => setJudgeModel(e.target.value)}
                className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none border border-border/60"
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={!prompt.trim() || running}
            className="btn-primary !min-h-[36px] !px-4 text-xs disabled:opacity-50 flex items-center gap-1.5"
          >
            <Icons.Goals className="w-4 h-4" />
            <span>{running ? "Menjalankan..." : "Jalankan Misi"}</span>
          </button>
        </form>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-bold text-text">Riwayat Misi</h3>
          <button
            onClick={loadHistory}
            className="btn-soft !min-h-[32px] !py-1 !px-3 text-xs flex items-center gap-1.5"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Segarkan</span>
          </button>
        </div>

        <div className="table-scroll neu-card border border-border/60">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border/60 bg-surface text-text-secondary">
                <th className="p-3 font-semibold">ID Goal</th>
                <th className="p-3 font-semibold">Objektif</th>
                <th className="p-3 font-semibold">Status</th>
                <th className="p-3 font-semibold">Putusan</th>
                <th className="p-3 font-semibold tabular-nums">Skor</th>
                <th className="p-3 font-semibold text-right">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={6} className="p-8 text-center text-text-secondary">
                    Memuat riwayat...
                  </td>
                </tr>
              ) : goals.length === 0 ? (
                <tr>
                  <td colSpan={6} className="p-8 text-center text-text-secondary">
                    Belum ada misi dijalankan
                  </td>
                </tr>
              ) : (
                (Array.isArray(goals) ? goals : []).map((g) => (
                  <tr
                    key={g.goal_id}
                    className="border-b border-border/40 hover:bg-surface-inset/40 transition-colors"
                  >
                    <td className="p-3 font-mono font-medium text-text">
                      {g.goal_id.substring(0, 10)}...
                    </td>
                    <td className="p-3 max-w-xs truncate text-text">{g.prompt}</td>
                    <td className="p-3">
                      <span className="badge-status bg-surface-inset text-text-secondary text-[10px]">
                        {g.status}
                      </span>
                    </td>
                    <td className="p-3">
                      {g.verdict === "SUCCESS" ? (
                        <span className="badge-status bg-success/15 text-success text-[10px]">
                          SUKSES
                        </span>
                      ) : g.verdict === "FAILURE" ? (
                        <span className="badge-status bg-danger/15 text-danger text-[10px]">
                          GAGAL
                        </span>
                      ) : (
                        <span className="badge-status bg-warning/15 text-warning text-[10px]">
                          PROSES
                        </span>
                      )}
                    </td>
                    <td className="p-3 font-mono tabular-nums text-text">
                      {g.score !== undefined ? g.score.toFixed(2) : "-"}
                    </td>
                    <td className="p-3 text-right">
                      <button
                        onClick={() => setSelectedGoal(g)}
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
      </div>

      {selectedGoal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-sm p-4">
          <div className="neu-focal p-5 w-full max-w-lg relative max-h-[85vh] overflow-y-auto border border-border/60">
            <div className="flex items-center justify-between pb-3 border-b border-border/60 mb-4">
              <h3 className="text-sm font-bold text-text">Detail & Jejak Juri</h3>
              <button
                onClick={() => setSelectedGoal(null)}
                className="p-1 text-text-secondary hover:text-text rounded-md hover:bg-surface-inset"
              >
                <Icons.X className="w-4 h-4" />
              </button>
            </div>

            <div className="space-y-3 text-xs">
              <div>
                <span className="text-text-secondary">ID Goal: </span>
                <span className="font-mono font-bold text-text">{selectedGoal.goal_id}</span>
              </div>
              <div>
                <span className="text-text-secondary">Objektif: </span>
                <p className="mt-1 p-2 rounded-lg bg-surface-inset border border-border/60 text-text">
                  {selectedGoal.prompt}
                </p>
              </div>
              <div className="grid grid-cols-2 gap-3 p-3 rounded-xl bg-surface-inset border border-border/60">
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Putusan</div>
                  <div className="font-bold text-sm text-text">{selectedGoal.verdict || "PENDING"}</div>
                </div>
                <div>
                  <div className="text-[10px] text-text-secondary uppercase">Skor</div>
                  <div className="font-mono font-bold text-sm text-text">
                    {selectedGoal.score !== undefined ? selectedGoal.score.toFixed(2) : "-"}
                  </div>
                </div>
              </div>

              {selectedGoal.critique && (
                <div>
                  <div className="text-text-secondary font-semibold mb-1">Evaluasi Juri:</div>
                  <div className="p-3 rounded-xl bg-surface-inset border border-border/60 text-text whitespace-pre-wrap">
                    {selectedGoal.critique}
                  </div>
                </div>
              )}

              {Array.isArray(selectedGoal.trace) && selectedGoal.trace.length > 0 && (
                <div>
                  <div className="text-text-secondary font-semibold mb-1">Jejak Keputusan:</div>
                  <div className="space-y-1.5 max-h-48 overflow-y-auto font-mono text-[11px] p-2 rounded-xl bg-surface-inset border border-border/60">
                    {selectedGoal.trace.map((t, idx) => (
                      <div key={idx} className="text-text-secondary border-b border-border/30 pb-1">
                        {t}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
