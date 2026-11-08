"use client";

import React, { useState, useEffect } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { SessionInfo } from "@/types/api";

export default function SessionsPage() {
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterChannel, setFilterChannel] = useState("all");
  const [search, setSearch] = useState("");

  const loadSessions = async () => {
    setLoading(true);
    try {
      const data = await api.sessions();
      setSessions(Array.isArray(data) ? data : []);
    } catch {} finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSessions();
  }, []);

  const filtered = (Array.isArray(sessions) ? sessions : []).filter((s) => {
    const matchChan = filterChannel === "all" || s.channel === filterChannel;
    const matchSearch = s.session_id ? s.session_id.toLowerCase().includes(search.toLowerCase()) : false;
    return matchChan && matchSearch;
  });

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Sesi</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Daftar percakapan aktif dan riwayat channel.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadSessions}
            className="btn-soft !min-h-[34px] !py-1 !px-3 text-xs flex items-center gap-1.5"
            title="Segarkan Sesi"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Segarkan</span>
          </button>
          <Link
            href="/chat"
            className="btn-primary !min-h-[34px] !py-1 !px-3 text-xs flex items-center gap-1.5"
          >
            <Icons.Plus className="w-3.5 h-3.5" />
            <span>Chat Baru</span>
          </Link>
        </div>
      </div>

      <div className="flex flex-col sm:flex-row items-center gap-3">
        <div className="w-full sm:w-64">
          <input
            type="text"
            placeholder="Cari ID sesi..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="neu-inset w-full px-3 py-2 text-xs placeholder:text-text-secondary/50 focus:outline-none border border-border/60"
          />
        </div>

        <div className="flex items-center gap-1.5 w-full sm:w-auto overflow-x-auto pb-1 sm:pb-0">
          {[
            { id: "all", label: "Semua" },
            { id: "webui", label: "WebUI" },
            { id: "telegram", label: "Telegram" },
            { id: "discord", label: "Discord" },
            { id: "whatsapp", label: "WhatsApp" },
          ].map((ch) => (
            <button
              key={ch.id}
              onClick={() => setFilterChannel(ch.id)}
              className={`px-3 py-1.5 rounded-full text-xs font-semibold transition-all shrink-0 ${
                filterChannel === ch.id
                  ? "bg-primary text-white shadow-raised-sm"
                  : "bg-surface-raised border border-border/60 text-text-secondary hover:text-text"
              }`}
            >
              {ch.label}
            </button>
          ))}
        </div>
      </div>

      <div className="table-scroll neu-card border border-border/60">
        <table className="w-full text-left text-xs border-collapse">
          <thead>
            <tr className="border-b border-border/60 bg-surface text-text-secondary">
              <th className="p-3 font-semibold">ID Sesi</th>
              <th className="p-3 font-semibold">Channel</th>
              <th className="p-3 font-semibold tabular-nums">Turn</th>
              <th className="p-3 font-semibold">Terakhir Aktif</th>
              <th className="p-3 font-semibold text-right">Aksi</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={5} className="p-8 text-center text-text-secondary">
                  Memuat sesi...
                </td>
              </tr>
            ) : filtered.length === 0 ? (
              <tr>
                <td colSpan={5} className="p-8 text-center text-text-secondary">
                  Tidak ada sesi yang cocok
                </td>
              </tr>
            ) : (
              (Array.isArray(filtered) ? filtered : []).map((s) => (
                <tr
                  key={s.session_id}
                  className="border-b border-border/40 hover:bg-surface-inset/40 transition-colors"
                >
                  <td className="p-3 font-mono font-medium text-text">{s.session_id}</td>
                  <td className="p-3">
                    <span className="px-2 py-0.5 rounded-full bg-primary/10 text-primary font-semibold text-[10px] uppercase">
                      {s.channel || "webui"}
                    </span>
                  </td>
                  <td className="p-3 font-mono tabular-nums text-text">
                    {s.turn_count || 0}
                  </td>
                  <td className="p-3 text-text-secondary">
                    {s.last_active ? new Date(s.last_active).toLocaleString() : "-"}
                  </td>
                  <td className="p-3 text-right">
                    <Link
                      href="/chat"
                      className="inline-flex items-center gap-1 text-primary hover:underline font-semibold"
                    >
                      <span>Buka</span>
                      <Icons.ChevronRight className="w-3.5 h-3.5" />
                    </Link>
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
