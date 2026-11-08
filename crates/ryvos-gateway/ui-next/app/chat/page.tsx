"use client";

import React, { useState, useEffect, useRef } from "react";
import Link from "next/link";
import { useWebSocket } from "@/lib/context";
import { api } from "@/lib/api";
import { wsClient } from "@/lib/ws";
import { Icons } from "@/components/icons/Icons";
import { MarkdownView } from "@/components/common/MarkdownView";
import { ThinkingFold } from "@/components/common/ThinkingFold";
import { ToolCallCard } from "@/components/common/ToolCallCard";
import { ApprovalCard } from "@/components/common/ApprovalCard";
import { ModelSelectorModal } from "@/components/common/ModelSelectorModal";
import type { SessionInfo, ChatMessage, ToolCallItem, ApprovalRequest } from "@/types/api";

export default function ChatPage() {
  const { status, streamingText, modelInfo, sendWs, resetStreamingText } = useWebSocket();
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string>("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputMessage, setInputMessage] = useState("");
  const [isRunning, setIsRunning] = useState(false);
  const [activeTools, setActiveTools] = useState<ToolCallItem[]>([]);
  const [approvals, setApprovals] = useState<ApprovalRequest[]>([]);
  const [showModelModal, setShowModelModal] = useState(false);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [currentKeys, setCurrentKeys] = useState<string[]>([]);
  const [selectedProvider, setSelectedProvider] = useState(modelInfo?.provider || "gemini");
  const [selectedModel, setSelectedModel] = useState(modelInfo?.model_id || "gemini-2.5-flash");
  const [sessionSearch, setSessionSearch] = useState("");
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadSessions();
    loadApprovals();
    loadModelConfig();
  }, []);

  useEffect(() => {
    if (modelInfo?.provider) {
      setSelectedProvider(modelInfo.provider);
      setSelectedModel(modelInfo.model_id);
    }
  }, [modelInfo]);

  const loadModelConfig = async () => {
    try {
      const data = await api.model.get();
      if (data) {
        if (data.provider) setSelectedProvider(data.provider);
        if (data.model_id) setSelectedModel(data.model_id);
        if (data.api_keys) setCurrentKeys(data.api_keys);
      }
    } catch {}
  };

  const getUrlSessionId = (): string | null => {
    if (typeof window === "undefined") return null;
    try {
      const sp = new URLSearchParams(window.location.search);
      return sp.get("id");
    } catch {
      return null;
    }
  };

  const updateUrlSessionId = (id: string) => {
    if (typeof window === "undefined") return;
    try {
      const url = new URL(window.location.href);
      url.searchParams.set("id", id);
      window.history.replaceState(null, "", url.toString());
    } catch {}
  };

  const loadSessions = async () => {
    try {
      const list = await api.sessions();
      const safeList = Array.isArray(list) ? list : [];
      setSessions(safeList);
      const urlId = getUrlSessionId();
      if (urlId) {
        selectSession(urlId);
      } else if (!currentSessionId && safeList.length > 0) {
        selectSession(safeList[0].session_id);
      } else if (!currentSessionId) {
        createNewSession();
      }
    } catch {
      const urlId = getUrlSessionId();
      if (urlId) {
        selectSession(urlId);
      } else if (!currentSessionId) {
        createNewSession();
      }
    }
  };

  const loadApprovals = async () => {
    try {
      const list = await api.approvals.list();
      setApprovals(Array.isArray(list) ? list : []);
    } catch {}
  };

  const createNewSession = () => {
    const newId =
      typeof crypto !== "undefined" && crypto.randomUUID
        ? crypto.randomUUID()
        : `ses_${Date.now().toString(36)}`;
    setCurrentSessionId(newId);
    updateUrlSessionId(newId);
    setMessages([]);
    setActiveTools([]);
    resetStreamingText();
  };

  const selectSession = async (id: string) => {
    setCurrentSessionId(id);
    updateUrlSessionId(id);
    setActiveTools([]);
    resetStreamingText();
    try {
      const data = await api.sessionHistory(id);
      if (data && Array.isArray(data.messages)) {
        setMessages(
          data.messages
            .filter((m: any) => m.content || m.text || m.message)
            .map((m: any) => ({
              role: m.role || "user",
              content: m.content || m.text || m.message || "",
              timestamp: m.timestamp,
            }))
        );
      } else {
        setMessages([]);
      }
    } catch {
      setMessages([]);
    }
  };

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText, activeTools]);

  useEffect(() => {
    const unsub = wsClient.onRawEvent((event, sessId) => {
      const matchesSession =
        !sessId ||
        !currentSessionId ||
        sessId === currentSessionId ||
        currentSessionId.includes(sessId) ||
        sessId.includes(currentSessionId);
      if (!matchesSession) return;

      if (event.kind === "run_started") {
        setIsRunning(true);
      } else if (event.kind === "run_complete") {
        setIsRunning(false);
        const finalStream = wsClient.streamingText;
        if (finalStream) {
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last?.role === "assistant" && last.content === finalStream) return prev;
            return [
              ...prev,
              {
                role: "assistant",
                content: finalStream,
                timestamp: new Date().toISOString(),
              },
            ];
          });
        }
        resetStreamingText();
        loadSessions();
      } else if (event.kind === "run_error") {
        setIsRunning(false);
        const err = event.data?.error || "Agent execution error";
        setMessages((prev) => [
          ...prev,
          {
            role: "assistant",
            content: `Error: ${err}`,
            timestamp: new Date().toISOString(),
          },
        ]);
        resetStreamingText();
      } else if (event.kind === "tool_start") {
        const item: ToolCallItem = {
          id: String(Date.now() + Math.random()),
          name: event.tool || "tool",
          input: typeof event.input === "string" ? event.input : JSON.stringify(event.input || {}, null, 2),
          status: "running",
        };
        setActiveTools((prev) => [...(Array.isArray(prev) ? prev : []), item]);
      } else if (event.kind === "tool_end") {
        setActiveTools((prev) =>
          (Array.isArray(prev) ? prev : []).map((t) =>
            t.name === event.tool && t.status === "running"
              ? {
                  ...t,
                  output:
                    typeof event.output === "string" ? event.output : JSON.stringify(event.output || {}, null, 2),
                  status: "completed",
                  elapsed_ms: event.data?.elapsed_ms,
                }
              : t
          )
        );
      } else if (event.kind === "approval_requested") {
        loadApprovals();
      }
    });

    return unsub;
  }, [currentSessionId, resetStreamingText]);

  const handleSendMessage = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!inputMessage.trim() || isRunning) return;

    const userText = inputMessage.trim();
    setInputMessage("");

    setMessages((prev) => [
      ...prev,
      {
        role: "user",
        content: userText,
        timestamp: new Date().toISOString(),
      },
    ]);

    resetStreamingText();
    setActiveTools([]);
    setIsRunning(true);

    try {
      const res = await sendWs("agent.send", {
        session_id: currentSessionId,
        message: userText,
        provider: selectedProvider,
        model_id: selectedModel,
      });

      if (res?.session_id && res.session_id !== currentSessionId) {
        setCurrentSessionId(res.session_id);
        updateUrlSessionId(res.session_id);
      }

      if (res?.response) {
        setMessages((prev) => {
          const last = prev[prev.length - 1];
          if (last?.role === "assistant" && last.content === res.response) {
            return prev;
          }
          return [
            ...prev,
            {
              role: "assistant",
              content: res.response,
              timestamp: new Date().toISOString(),
            },
          ];
        });
      }
      setIsRunning(false);
      resetStreamingText();
      loadSessions();
    } catch (err: any) {
      setIsRunning(false);
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: `Error: ${err.message || err}`,
          timestamp: new Date().toISOString(),
        },
      ]);
      resetStreamingText();
    }
  };

  const handleStopRun = async () => {
    try {
      await sendWs("agent.cancel", { session_id: currentSessionId });
    } catch {}
    setIsRunning(false);
  };

  const handleApplyModel = async (provider: string, modelId: string, keys: string[]) => {
    setSelectedProvider(provider);
    setSelectedModel(modelId);
    setCurrentKeys(keys);

    try {
      await sendWs("model.set", {
        provider,
        model_id: modelId,
        api_keys: keys,
      });
    } catch {}

    try {
      await api.model.put({
        provider,
        model_id: modelId,
        api_keys: keys,
      });
    } catch {}
  };

  const parseThinkContent = (content: string) => {
    const thinkMatch = content.match(/<think>([\s\S]*?)<\/think>/);
    if (thinkMatch) {
      const think = thinkMatch[1].trim();
      const rest = content.replace(/<think>[\s\S]*?<\/think>/, "").trim();
      return { think, body: rest };
    }
    return { think: "", body: content };
  };

  const handleCopyMessage = (text: string, idx: number) => {
    navigator.clipboard.writeText(text);
    setCopiedIdx(idx);
    setTimeout(() => setCopiedIdx(null), 1800);
  };

  const filteredSessions = (Array.isArray(sessions) ? sessions : []).filter((s) =>
    s && s.session_id && s.session_id.toLowerCase().includes(sessionSearch.toLowerCase())
  );

  return (
    <div className="flex-1 flex flex-col h-full w-full relative overflow-hidden bg-bg">
      <div className="px-4 py-2 border-b border-border/60 bg-surface/60 backdrop-blur-md flex items-center justify-between text-xs max-w-4xl mx-auto w-full shrink-0">
        <button
          type="button"
          onClick={() => setDrawerOpen(true)}
          className="flex items-center gap-1.5 text-text-secondary hover:text-text font-mono truncate max-w-[220px]"
          title="Buka daftar sesi"
        >
          <Icons.Sessions className="w-3.5 h-3.5 text-primary shrink-0" />
          <span className="truncate">{currentSessionId || "Sesi Obrolan"}</span>
        </button>
        <button
          type="button"
          onClick={createNewSession}
          className="btn-soft !min-h-[26px] !py-0.5 !px-2 text-[11px] flex items-center gap-1"
          title="Mulai sesi baru"
        >
          <Icons.Plus className="w-3 h-3" />
          <span>Baru</span>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-4 md:px-8 space-y-4 max-w-4xl mx-auto w-full no-scrollbar">
        {messages.length === 0 && !streamingText && (
          <div className="flex flex-col items-center justify-center min-h-[300px] text-center text-text-secondary my-auto">
            <div className="w-12 h-12 rounded-2xl bg-surface-raised border border-border/60 flex items-center justify-center text-primary mb-3 shadow-raised-sm">
              <Icons.Chat className="w-6 h-6" />
            </div>
            <h3 className="text-base font-bold text-text mb-1">Ryvos AI</h3>
            <p className="text-xs text-text-secondary max-w-xs">
              Mulai percakapan atau ketik instruksi di bawah
            </p>
          </div>
        )}

        {(Array.isArray(approvals) ? approvals : []).map((req) => (
          <div key={req.id} className="max-w-[95%] sm:max-w-[85%] mr-auto w-full">
            <ApprovalCard approval={req} onResolved={() => loadApprovals()} />
          </div>
        ))}

        {(Array.isArray(messages) ? messages : []).map((m, idx) => {
          if (!m.content || !m.content.trim()) return null;
          const isUser = m.role === "user";
          const { think, body } = parseThinkContent(m.content);

          return (
            <div key={idx} className="w-full">
              {isUser ? (
                <div className="max-w-[85%] sm:max-w-[70%] ml-auto bg-primary text-white rounded-2xl rounded-tr-sm px-4 py-3 text-sm shadow-raised-sm whitespace-pre-wrap break-words">
                  {m.content}
                </div>
              ) : (
                <div className="max-w-[95%] sm:max-w-[85%] mr-auto bg-surface-raised border border-border/50 text-text rounded-2xl rounded-tl-sm px-4 py-3.5 text-sm shadow-raised-sm break-words relative group">
                  {think && <ThinkingFold think={think} />}
                  <MarkdownView content={body} />
                  <div className="mt-2 pt-1 border-t border-border/30 flex items-center justify-end">
                    <button
                      type="button"
                      onClick={() => handleCopyMessage(body, idx)}
                      className="p-1 rounded text-text-secondary hover:text-text text-xs flex items-center gap-1 transition-colors"
                      title="Salin pesan"
                    >
                      {copiedIdx === idx ? (
                        <>
                          <Icons.Check className="w-3.5 h-3.5 text-success" />
                          <span className="text-[11px] text-success">Tersalin</span>
                        </>
                      ) : (
                        <>
                          <Icons.Copy className="w-3.5 h-3.5" />
                          <span className="text-[11px]">Salin</span>
                        </>
                      )}
                    </button>
                  </div>
                </div>
              )}
            </div>
          );
        })}

        {Array.isArray(activeTools) && activeTools.length > 0 && (
          <div className="max-w-[95%] sm:max-w-[85%] mr-auto w-full space-y-2">
            {activeTools.map((tool) => (
              <ToolCallCard key={tool.id} tool={tool} />
            ))}
          </div>
        )}

        {streamingText && (
          <div className="max-w-[95%] sm:max-w-[85%] mr-auto bg-surface-raised border border-border/50 text-text rounded-2xl rounded-tl-sm px-4 py-3.5 text-sm shadow-raised-sm break-words">
            {(() => {
              const { think, body } = parseThinkContent(streamingText);
              return (
                <>
                  {think && <ThinkingFold think={think} />}
                  <MarkdownView content={body} />
                  <span className="streaming-cursor" />
                </>
              );
            })()}
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <div className="p-3 md:p-4 max-w-4xl mx-auto w-full shrink-0">
        <form
          onSubmit={handleSendMessage}
          className="neu-inset rounded-2xl p-2.5 bg-surface-inset border border-border/60 focus-within:border-primary/50 transition-all flex flex-col gap-2"
        >
          <textarea
            value={inputMessage}
            onChange={(e) => setInputMessage(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSendMessage();
              }
            }}
            rows={2}
            placeholder=""
            className="w-full bg-transparent text-sm text-text placeholder:text-text-secondary/50 resize-none focus:outline-none border-none p-1"
          />

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setDrawerOpen(true)}
                className="w-7 h-7 rounded-full bg-surface-raised border border-border/60 flex items-center justify-center text-text-secondary hover:text-text hover:border-primary/50 shadow-raised-sm transition-all"
                title="Buka Sesi"
              >
                <Icons.Plus className="w-3.5 h-3.5" />
              </button>

              <button
                type="button"
                onClick={() => setShowModelModal(true)}
                className="px-2.5 py-1 rounded-full bg-surface-raised border border-border/60 text-xs font-medium text-text flex items-center gap-1.5 hover:border-primary/50 shadow-raised-sm"
                title="Pilih Model"
              >
                <span className="truncate max-w-[140px] sm:max-w-[200px]">
                  {selectedModel || "Pilih Model"}
                </span>
                <Icons.ChevronUp className="w-3 h-3 text-text-secondary shrink-0" />
              </button>

              <Link
                href="/keys"
                className="px-2 py-1 rounded-full text-xs font-semibold text-text-secondary hover:text-primary hover:bg-surface-raised border border-border/60 flex items-center gap-1 shadow-raised-sm"
                title="Kelola API Key & Provider"
              >
                <Icons.Key className="w-3.5 h-3.5 text-primary" />
                <span className="hidden sm:inline">Keys</span>
              </Link>
            </div>

            <div>
              {isRunning ? (
                <button
                  type="button"
                  onClick={handleStopRun}
                  className="w-9 h-9 rounded-full bg-danger hover:bg-danger-strong text-white flex items-center justify-center shadow-raised-sm transition-all"
                  title="Stop"
                >
                  <Icons.Square className="w-4 h-4 fill-current" />
                </button>
              ) : (
                <button
                  type="submit"
                  disabled={!inputMessage.trim()}
                  className="w-9 h-9 rounded-full bg-primary hover:bg-primary-strong text-white flex items-center justify-center shadow-raised-sm disabled:opacity-40 disabled:pointer-events-none transition-all"
                  title="Kirim"
                >
                  <Icons.ArrowRight className="w-4 h-4 stroke-[2.5]" />
                </button>
              )}
            </div>
          </div>
        </form>
      </div>

      {drawerOpen && (
        <div className="fixed inset-0 z-40 bg-bg/60 backdrop-blur-sm flex">
          <div className="w-72 sm:w-80 bg-surface border-r border-border/60 shadow-floating p-4 flex flex-col h-full animate-fade-in">
            <div className="flex items-center justify-between pb-3 border-b border-border/60">
              <h2 className="text-sm font-bold text-text">Sesi Obrolan</h2>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => {
                    createNewSession();
                    setDrawerOpen(false);
                  }}
                  className="btn-soft !min-h-[28px] !py-1 !px-2 text-xs flex items-center gap-1"
                  title="Sesi Baru"
                >
                  <Icons.Plus className="w-3 h-3" />
                  <span>Baru</span>
                </button>
                <button
                  onClick={() => setDrawerOpen(false)}
                  className="p-1 rounded text-text-secondary hover:text-text"
                  title="Tutup"
                >
                  <Icons.X className="w-4 h-4" />
                </button>
              </div>
            </div>

            <div className="my-2">
              <input
                type="text"
                placeholder="Cari sesi..."
                value={sessionSearch}
                onChange={(e) => setSessionSearch(e.target.value)}
                className="neu-inset w-full px-2.5 py-1.5 text-xs placeholder:text-text-secondary/50 focus:outline-none border border-border/60 rounded-lg"
              />
            </div>

            <div className="flex-1 overflow-y-auto space-y-1.5 pr-1 no-scrollbar">
              {filteredSessions.length === 0 ? (
                <div className="text-center py-6 text-xs text-text-secondary">Tidak ada sesi</div>
              ) : (
                filteredSessions.map((s) => {
                  const active = s.session_id === currentSessionId;
                  return (
                    <button
                      key={s.session_id}
                      onClick={() => {
                        selectSession(s.session_id);
                        setDrawerOpen(false);
                      }}
                      className={`w-full p-2.5 rounded-xl text-left transition-all ${
                        active
                          ? "bg-primary-soft text-primary font-semibold shadow-raised-sm border border-primary/25"
                          : "bg-surface text-text hover:bg-surface-inset border border-transparent"
                      }`}
                    >
                      <div className="flex items-center justify-between gap-1 text-xs">
                        <span className="font-mono truncate">{s.session_id}</span>
                        <span className="text-[10px] text-text-secondary shrink-0 font-mono">
                          {s.turn_count || 0} turn
                        </span>
                      </div>
                      <div className="text-[10px] text-text-secondary mt-1 flex items-center justify-between">
                        <span className="uppercase">{s.channel || "webui"}</span>
                        <span>{s.last_active ? new Date(s.last_active).toLocaleDateString() : ""}</span>
                      </div>
                    </button>
                  );
                })
              )}
            </div>
          </div>
          <div className="flex-1" onClick={() => setDrawerOpen(false)} />
        </div>
      )}

      <ModelSelectorModal
        isOpen={showModelModal}
        onClose={() => setShowModelModal(false)}
        currentProvider={selectedProvider}
        currentModel={selectedModel}
        currentKeys={currentKeys}
        onApply={handleApplyModel}
      />
    </div>
  );
}
