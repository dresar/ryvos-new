"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { CronJob } from "@/types/api";

export default function CronGraphPage() {
  const [jobs, setJobs] = useState<CronJob[]>([]);
  const [heartbeats, setHeartbeats] = useState<any[]>([]);
  const [skills, setSkills] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [jobName, setJobName] = useState("");
  const [cronExpr, setCronExpr] = useState("0 */4 * * *");
  const [prompt, setPrompt] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const loadAll = async () => {
    setLoading(true);
    try {
      const [j, hb, sk] = await Promise.allSettled([
        api.cron.list(),
        api.heartbeatHistory(),
        api.skills(),
      ]);
      if (j.status === "fulfilled") setJobs(Array.isArray(j.value) ? j.value : []);
      if (hb.status === "fulfilled") setHeartbeats(Array.isArray(hb.value) ? hb.value : (hb.value as any)?.events || []);
      if (sk.status === "fulfilled") setSkills(Array.isArray(sk.value?.skills) ? sk.value.skills : (Array.isArray(sk.value) ? sk.value : []));
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAll();
  }, []);

  const handleCreateJob = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!jobName.trim() || !cronExpr.trim() || !prompt.trim()) return;

    setSubmitting(true);
    try {
      await api.cron.add({
        name: jobName.trim(),
        expr: cronExpr.trim(),
        prompt: prompt.trim(),
      });
      setShowAddModal(false);
      setJobName("");
      setPrompt("");
      loadAll();
    } catch (err: any) {
      alert(`Failed to add cron job: ${err.message}`);
    } finally {
      setSubmitting(false);
    }
  };

  const handleDeleteJob = async (name: string) => {
    try {
      await api.cron.delete(name);
      loadAll();
    } catch (err: any) {
      alert(`Failed to delete cron job: ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Graph, Cron & Heartbeat</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Autonomous scheduled agent routines, runtime heartbeat health checks, and skill dependencies
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadAll}
            className="btn-soft !min-h-[36px] !py-1 text-xs"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Refresh</span>
          </button>
          <button
            onClick={() => setShowAddModal(true)}
            className="btn-primary !min-h-[36px] !py-1 text-xs"
          >
            <Icons.Plus className="w-3.5 h-3.5" />
            <span>New Cron Job</span>
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          <div className="neu-card p-5">
            <h3 className="text-sm font-bold text-text mb-3">Scheduled Cron Jobs</h3>
            <div className="table-scroll">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b border-border bg-surface text-text-secondary">
                    <th className="p-3 font-semibold">Name</th>
                    <th className="p-3 font-semibold">Cron Expression</th>
                    <th className="p-3 font-semibold">Prompt Trigger</th>
                    <th className="p-3 font-semibold text-right">Action</th>
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    <tr>
                      <td colSpan={4} className="p-6 text-center text-text-secondary">Loading jobs...</td>
                    </tr>
                  ) : jobs.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="p-6 text-center text-text-secondary">No cron jobs configured</td>
                    </tr>
                  ) : (
                    (Array.isArray(jobs) ? jobs : []).map((job) => (
                      <tr key={job.name} className="border-b border-border/50 hover:bg-surface-inset/50 transition-colors">
                        <td className="p-3 font-mono font-bold text-text">{job.name}</td>
                        <td className="p-3 font-mono text-primary">{job.expr}</td>
                        <td className="p-3 max-w-xs truncate text-text-secondary">{job.prompt}</td>
                        <td className="p-3 text-right">
                          <button
                            onClick={() => handleDeleteJob(job.name)}
                            className="text-danger hover:underline font-semibold"
                          >
                            Delete
                          </button>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>

          <div className="neu-card p-5">
            <h3 className="text-sm font-bold text-text mb-3">Heartbeat Health Logs</h3>
            <div className="space-y-2 max-h-60 overflow-y-auto no-scrollbar font-mono text-xs">
              {heartbeats.length === 0 ? (
                <div className="text-center py-6 text-text-secondary">No recent heartbeat pings recorded</div>
              ) : (
                (Array.isArray(heartbeats) ? heartbeats : []).map((hb, i) => (
                  <div key={i} className="p-2.5 rounded bg-surface-inset border border-border flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="w-2 h-2 rounded-full bg-success" />
                      <span className="text-text">{hb.message || "Heartbeat Ping OK"}</span>
                    </div>
                    <span className="text-[10px] text-text-secondary">{hb.time || new Date().toLocaleTimeString()}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div>
          <div className="neu-card p-5 space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-border">
              <h3 className="text-sm font-bold text-text">Installed Skills</h3>
              <span className="text-xs font-mono text-text-secondary">{skills.length}</span>
            </div>

            <div className="space-y-2 max-h-[480px] overflow-y-auto no-scrollbar">
              {skills.length === 0 ? (
                <div className="text-center py-8 text-xs text-text-secondary">No custom skills loaded</div>
              ) : (
                (Array.isArray(skills) ? skills : []).map((skill, idx) => (
                  <div key={idx} className="p-2.5 rounded-lg bg-surface-inset border border-border flex items-center gap-2 text-xs">
                    <Icons.Wrench className="w-3.5 h-3.5 text-primary shrink-0" />
                    <span className="font-mono text-text truncate">{skill}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {showAddModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-sm p-4">
          <div className="neu-focal p-6 w-full max-w-md relative">
            <div className="flex items-center justify-between pb-3 border-b border-border mb-4">
              <h3 className="text-sm font-bold text-text">Schedule Autonomous Cron Job</h3>
              <button
                onClick={() => setShowAddModal(false)}
                className="p-1 text-text-secondary hover:text-text"
              >
                <Icons.X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCreateJob} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">Job Name</label>
                <input
                  type="text"
                  placeholder="e.g. nightly-security-audit"
                  value={jobName}
                  onChange={(e) => setJobName(e.target.value)}
                  className="neu-inset w-full px-3 py-2 text-xs focus:outline-none"
                  required
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">Standard 5-Field Cron Expression</label>
                <input
                  type="text"
                  placeholder="0 */4 * * *"
                  value={cronExpr}
                  onChange={(e) => setCronExpr(e.target.value)}
                  className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none"
                  required
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">Execution Prompt / Trigger</label>
                <textarea
                  rows={3}
                  placeholder="Instructions for the agent to run on schedule..."
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  className="neu-inset w-full p-2.5 text-xs focus:outline-none"
                  required
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="btn-outline !min-h-[36px] !py-1 text-xs"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={submitting}
                  className="btn-primary !min-h-[36px] !py-1 text-xs"
                >
                  {submitting ? "Scheduling..." : "Schedule Job"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
