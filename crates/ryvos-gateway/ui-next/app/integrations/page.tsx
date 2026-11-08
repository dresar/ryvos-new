"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { IntegrationItem } from "@/types/api";

const STATIC_MCP_SERVERS = [
  {
    id: "supermemory",
    name: "Supermemory MCP",
    type: "Streamable HTTP",
    status: "connected",
    description: "Cloud long-term semantic memory, graph recall, and profile document sync.",
    tools_count: 9,
  },
  {
    id: "agentpack",
    name: "AgentPack MCP",
    type: "stdio",
    status: "connected",
    description: "Multi-agent packing, token efficiency compression, and task boundary analysis.",
    tools_count: 31,
  },
  {
    id: "mizan",
    name: "Mizan MCP",
    type: "stdio",
    status: "connected",
    description: "AST-based command safety gate and zeroized credential policy auditor.",
    tools_count: 11,
  },
];

export default function IntegrationsPage() {
  const [tab, setTab] = useState<"oauth" | "mcp">("oauth");
  const [items, setItems] = useState<IntegrationItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [serverName, setServerName] = useState("");
  const [serverType, setServerType] = useState("stdio");
  const [commandOrUrl, setCommandOrUrl] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const loadIntegrations = async () => {
    setLoading(true);
    try {
      const data = await api.integrations.list();
      setItems(Array.isArray(data) ? data : []);
    } catch {
      setItems([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadIntegrations();
  }, []);

  const handleConnectOAuth = async (appId: string) => {
    try {
      const res = await api.integrations.connect(appId, {});
      if (res && (res as any).redirect_url) {
        window.location.href = (res as any).redirect_url;
      } else if (res && (res as any).connected) {
        alert(`${appId} berhasil dihubungkan.`);
        loadIntegrations();
      } else if (res && (res as any).error) {
        alert(`Konfigurasi belum lengkap: ${(res as any).error}`);
      }
    } catch (err: any) {
      alert(`Gagal menghubungkan: ${err.message}`);
    }
  };

  const handleDisconnect = async (id: string) => {
    try {
      await api.integrations.disconnect(id);
      alert(`${id} berhasil diputus.`);
      loadIntegrations();
    } catch (err: any) {
      alert(`Gagal memutuskan: ${err.message}`);
    }
  };

  const handleConnectMcp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!serverName.trim()) return;

    setSubmitting(true);
    try {
      await api.integrations.connect(serverName.trim(), {
        type: serverType,
        target: commandOrUrl.trim(),
      });
      setShowAddModal(false);
      setServerName("");
      setCommandOrUrl("");
      alert(`Server MCP ${serverName} berhasil ditambahkan.`);
    } catch (err: any) {
      alert(`Gagal menambahkan server MCP: ${err.message}`);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">Integrasi & Extensions</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Konektor OAuth cloud, Model Context Protocol servers, dan runtime tool extensions.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={loadIntegrations}
            className="btn-soft !min-h-[36px] !py-1 text-xs"
            title="Muat ulang status"
          >
            <Icons.Refresh className="w-3.5 h-3.5" />
            <span>Segarkan</span>
          </button>
          <button
            onClick={() => setShowAddModal(true)}
            className="btn-primary !min-h-[36px] !py-1 text-xs"
          >
            <Icons.Plus className="w-3.5 h-3.5" />
            <span>Tambah MCP Server</span>
          </button>
        </div>
      </div>

      <div className="flex items-center gap-2 p-1 bg-surface-inset rounded-xl border border-border/60 w-fit">
        <button
          type="button"
          onClick={() => setTab("oauth")}
          className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            tab === "oauth"
              ? "bg-primary text-white shadow-raised-sm"
              : "text-text-secondary hover:text-text"
          }`}
        >
          Cloud & OAuth Apps ({items.length || 8})
        </button>
        <button
          type="button"
          onClick={() => setTab("mcp")}
          className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all ${
            tab === "mcp"
              ? "bg-primary text-white shadow-raised-sm"
              : "text-text-secondary hover:text-text"
          }`}
        >
          MCP Servers ({STATIC_MCP_SERVERS.length})
        </button>
      </div>

      {tab === "oauth" && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {loading ? (
            <div className="col-span-full neu-card p-8 text-center text-xs text-text-secondary">
              Memuat daftar integrasi...
            </div>
          ) : items.length === 0 ? (
            <div className="col-span-full neu-card p-8 text-center text-xs text-text-secondary">
              Tidak ada aplikasi OAuth yang tersedia.
            </div>
          ) : (
            items.map((item) => (
              <div key={item.id} className="neu-card p-5 flex flex-col justify-between">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <Icons.Integrations className="w-4 h-4 text-primary" />
                      <span className="font-bold text-sm text-text">{item.name}</span>
                    </div>
                    <span
                      className={`badge-status text-[10px] ${
                        item.connected
                          ? "bg-success/15 text-success"
                          : item.configured
                          ? "bg-warning/15 text-warning"
                          : "bg-surface-inset text-text-secondary"
                      }`}
                    >
                      {item.connected
                        ? "Terhubung"
                        : item.configured
                        ? "Siap OAuth"
                        : "Belum Dikonfigurasi"}
                    </span>
                  </div>

                  <div className="text-xs text-text-secondary mb-1">
                    Provider: <span className="font-semibold text-text capitalize">{item.provider || "oauth"}</span>
                  </div>
                  <div className="text-xs text-text-secondary mb-3">
                    Tools: <span className="font-mono text-text">{item.actions || 0} actions</span>
                  </div>
                </div>

                <div className="pt-3 border-t border-border/40 flex items-center justify-between">
                  <span className="text-[10px] text-text-secondary font-mono">
                    ID: {item.id}
                  </span>
                  {item.connected ? (
                    <button
                      type="button"
                      onClick={() => handleDisconnect(item.id)}
                      className="btn-soft !text-danger hover:!bg-danger/10 !min-h-[28px] !py-0.5 text-xs"
                    >
                      Putuskan
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => handleConnectOAuth(item.id)}
                      className="btn-soft !text-primary hover:!bg-primary/10 !min-h-[28px] !py-0.5 text-xs font-semibold"
                    >
                      Hubungkan
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {tab === "mcp" && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {STATIC_MCP_SERVERS.map((server) => (
            <div key={server.id} className="neu-card p-5 flex flex-col justify-between">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Icons.Viking className="w-4 h-4 text-primary" />
                    <span className="font-bold text-sm text-text">{server.name}</span>
                  </div>
                  <span className="badge-status text-[10px] bg-success/15 text-success">
                    {server.status}
                  </span>
                </div>

                <div className="text-xs text-text-secondary mb-2">
                  Transport: <span className="font-mono text-text">{server.type}</span>
                </div>
                <p className="text-xs text-text-secondary line-clamp-2 mb-3">
                  {server.description}
                </p>
              </div>

              <div className="pt-3 border-t border-border/40 flex items-center justify-between">
                <span className="text-[10px] text-text-secondary font-mono">
                  {server.tools_count} tools aktif
                </span>
                <span className="text-[11px] text-primary font-semibold">
                  Terkoneksi
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {showAddModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-sm p-4">
          <div className="neu-focal p-6 w-full max-w-md relative">
            <div className="flex items-center justify-between pb-3 border-b border-border/60 mb-4">
              <h3 className="text-sm font-bold text-text">Tambah MCP Server</h3>
              <button
                type="button"
                onClick={() => setShowAddModal(false)}
                className="p-1 text-text-secondary hover:text-text"
              >
                <Icons.X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleConnectMcp} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">
                  Nama Server
                </label>
                <input
                  type="text"
                  placeholder="mis. postgres-mcp, filesystem-mcp"
                  value={serverName}
                  onChange={(e) => setServerName(e.target.value)}
                  className="neu-inset w-full px-3 py-2 text-xs focus:outline-none"
                  required
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">
                  Protokol Transport
                </label>
                <select
                  value={serverType}
                  onChange={(e) => setServerType(e.target.value)}
                  className="neu-inset w-full px-3 py-2 text-xs focus:outline-none bg-surface-inset"
                >
                  <option value="stdio">Standard I/O (Subprocess command)</option>
                  <option value="sse">Server-Sent Events (HTTP SSE)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-text-secondary mb-1">
                  {serverType === "stdio" ? "Command / Binary Path" : "SSE Endpoint URL"}
                </label>
                <input
                  type="text"
                  placeholder={serverType === "stdio" ? "npx -y @modelcontextprotocol/server-postgres" : "http://localhost:8000/sse"}
                  value={commandOrUrl}
                  onChange={(e) => setCommandOrUrl(e.target.value)}
                  className="neu-inset w-full px-3 py-2 text-xs font-mono focus:outline-none"
                  required
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="btn-outline !min-h-[36px] !py-1 text-xs"
                >
                  Batal
                </button>
                <button
                  type="submit"
                  disabled={submitting}
                  className="btn-primary !min-h-[36px] !py-1 text-xs"
                >
                  {submitting ? "Menghubungkan..." : "Hubungkan Server"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
