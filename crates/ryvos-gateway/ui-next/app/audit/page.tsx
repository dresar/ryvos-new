"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type {
  AuditEntry,
  AuditStats,
  SafetyLesson,
  DecisionEntry,
  FailureEntry,
} from "@/types/api";

export default function AuditPage() {
  const [tab, setTab] = useState<"security" | "safety" | "decisions" | "failures">("security");
  const [stats, setStats] = useState<AuditStats | null>(null);
  const [events, setEvents] = useState<AuditEntry[]>([]);
  const [lessons, setLessons] = useState<SafetyLesson[]>([]);
  const [decisions, setDecisions] = useState<DecisionEntry[]>([]);
  const [failures, setFailures] = useState<FailureEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const loadAll = async () => {
    setLoading(true);
    try {
      const [s, ev, les, dec, fail] = await Promise.allSettled([
        api.auditStats(),
        api.audit(100),
        api.safetyLessons(),
        api.decisions(50),
        api.failures(),
      ]);
      if (s.status === "fulfilled") setStats(s.value);
      if (ev.status === "fulfilled") setEvents(Array.isArray(ev.value) ? ev.value : (ev.value as any)?.entries || []);
      if (les.status === "fulfilled") setLessons(Array.isArray(les.value) ? les.value : (les.value as any)?.lessons || []);
      if (dec.status === "fulfilled") setDecisions(Array.isArray(dec.value) ? dec.value : (dec.value as any)?.decisions || []);
      if (fail.status === "fulfilled") setFailures(Array.isArray(fail.value) ? fail.value : (fail.value as any)?.failures || []);
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAll();
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Audit & Observability Trail</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Security barrier enforcements, SSRF egress blocks, safety memories, and recovery journals
          </p>
        </div>

        <button
          onClick={loadAll}
          className="btn-soft !min-h-[36px] !py-1 text-xs self-start sm:self-auto"
        >
          <Icons.Refresh className="w-3.5 h-3.5" />
          <span>Refresh</span>
        </button>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">Security Events</div>
          <div className="text-lg font-bold font-mono text-text mt-1">{stats?.total_events || 0}</div>
        </div>
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">SSRF Blocks</div>
          <div className="text-lg font-bold font-mono text-danger mt-1">{stats?.ssrf_blocks || 0}</div>
        </div>
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">Tool Blocks</div>
          <div className="text-lg font-bold font-mono text-warning mt-1">{stats?.tool_blocks || 0}</div>
        </div>
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">Safety Lessons</div>
          <div className="text-lg font-bold font-mono text-primary mt-1">{lessons.length}</div>
        </div>
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">Decisions</div>
          <div className="text-lg font-bold font-mono text-text mt-1">{decisions.length}</div>
        </div>
        <div className="neu-card p-3 text-center">
          <div className="text-[10px] uppercase font-bold text-text-secondary">Failures</div>
          <div className="text-lg font-bold font-mono text-danger mt-1">{failures.length}</div>
        </div>
      </div>

      <div className="flex items-center gap-2 border-b border-border pb-2 overflow-x-auto">
        {(
          [
            { id: "security", label: "Security & SSRF" },
            { id: "safety", label: "Safety Lessons" },
            { id: "decisions", label: "Agent Decisions" },
            { id: "failures", label: "Failure Journal" },
          ] as const
        ).map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={`px-4 py-2 rounded-lg text-xs font-semibold whitespace-nowrap transition-all ${
              tab === t.id
                ? "bg-primary text-white shadow-raised-sm"
                : "text-text-secondary hover:text-text hover:bg-surface-inset"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === "security" && (
        <div className="table-scroll">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border bg-surface text-text-secondary">
                <th className="p-3 font-semibold">Timestamp</th>
                <th className="p-3 font-semibold">Category</th>
                <th className="p-3 font-semibold">Action</th>
                <th className="p-3 font-semibold">Details</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={4} className="p-8 text-center text-text-secondary">Loading audit events...</td>
                </tr>
              ) : events.length === 0 ? (
                <tr>
                  <td colSpan={4} className="p-8 text-center text-text-secondary">No security audit events recorded</td>
                </tr>
              ) : (
                (Array.isArray(events) ? events : []).map((e) => (
                  <tr key={e.id} className="border-b border-border/50 hover:bg-surface-inset/50 transition-colors">
                    <td className="p-3 font-mono text-text-secondary whitespace-nowrap">
                      {new Date(e.timestamp).toLocaleTimeString()}
                    </td>
                    <td className="p-3">
                      <span className="badge-status bg-surface-inset text-text uppercase text-[10px]">
                        {e.category}
                      </span>
                    </td>
                    <td className="p-3 font-mono font-medium text-text">{e.action}</td>
                    <td className="p-3 text-text-secondary max-w-md break-words">{e.detail}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {tab === "safety" && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {lessons.length === 0 ? (
            <div className="col-span-2 neu-card p-8 text-center text-text-secondary text-xs">
              No safety lessons learned yet
            </div>
          ) : (
            (Array.isArray(lessons) ? lessons : []).map((item) => (
              <div key={item.id} className="neu-card p-4 space-y-2">
                <div className="flex items-center justify-between text-xs font-mono text-text-secondary">
                  <span className="text-primary font-bold">{item.pattern}</span>
                  <span>{new Date(item.learned_at).toLocaleDateString()}</span>
                </div>
                <div className="text-xs font-semibold text-text">{item.action}</div>
                <div className="text-xs text-text-secondary leading-relaxed bg-surface-inset p-2.5 rounded border border-border">
                  {item.reason}
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {tab === "decisions" && (
        <div className="space-y-3">
          {decisions.length === 0 ? (
            <div className="neu-card p-8 text-center text-text-secondary text-xs">
              No autonomous decisions recorded yet
            </div>
          ) : (
            (Array.isArray(decisions) ? decisions : []).map((d) => (
              <div key={d.id} className="neu-card p-4 space-y-2 text-xs">
                <div className="flex items-center justify-between font-mono text-text-secondary">
                  <span className="font-bold text-text">{d.choice}</span>
                  <span>{new Date(d.timestamp).toLocaleTimeString()}</span>
                </div>
                <div className="text-text-secondary leading-relaxed">{d.justification}</div>
                {Array.isArray(d.alternatives) && d.alternatives.length > 0 && (
                  <div className="flex items-center gap-2 pt-1">
                    <span className="text-[10px] uppercase font-bold text-text-secondary">Alternatives:</span>
                    {d.alternatives.map((alt, i) => (
                      <span key={i} className="text-[10px] px-2 py-0.5 rounded bg-surface-inset border border-border">
                        {alt}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      {tab === "failures" && (
        <div className="space-y-3">
          {failures.length === 0 ? (
            <div className="neu-card p-8 text-center text-text-secondary text-xs">
              Zero runtime failures recorded. System running smoothly.
            </div>
          ) : (
            (Array.isArray(failures) ? failures : []).map((f) => (
              <div key={f.id} className="neu-card p-4 space-y-2 text-xs border-l-4 border-l-danger">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-danger font-mono">{f.error}</span>
                  <span className="text-text-secondary font-mono">{new Date(f.timestamp).toLocaleTimeString()}</span>
                </div>
                <div>
                  <span className="font-semibold text-text">Root Cause: </span>
                  <span className="text-text-secondary">{f.root_cause}</span>
                </div>
                <div className="p-2.5 rounded bg-surface-inset border border-border">
                  <span className="font-semibold text-text">Mitigation: </span>
                  <span className="text-text-secondary">{f.mitigation}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
