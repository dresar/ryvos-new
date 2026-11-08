import type {
  SessionInfo,
  MetricsData,
  RunRecord,
  CostRecord,
  AuditEntry,
  AuditStats,
  SafetyLesson,
  DecisionEntry,
  FailureEntry,
  VikingNode,
  VikingContent,
  ChannelStatus,
  IntegrationItem,
  CronJob,
  GoalRun,
  ModelInfo,
  ApprovalRequest,
} from "@/types/api";

let inMemoryApiKey = "";

export function getApiKey(): string {
  if (typeof window === "undefined") return inMemoryApiKey;
  return localStorage.getItem("ryvos_api_key") || inMemoryApiKey;
}

export function setApiKey(key: string): void {
  inMemoryApiKey = key;
  if (typeof window !== "undefined") {
    localStorage.setItem("ryvos_api_key", key);
  }
}

export function clearApiKey(): void {
  inMemoryApiKey = "";
  if (typeof window !== "undefined") {
    localStorage.removeItem("ryvos_api_key");
  }
}

export async function apiFetch<T = any>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string> | undefined),
  };
  const token = getApiKey();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(path, { ...options, headers });
  if (res.status === 401) {
    clearApiKey();
    if (typeof window !== "undefined") {
      window.dispatchEvent(new Event("ryvos:unauthorized"));
    }
    throw new Error("Unauthorized");
  }

  if (!res.ok) {
    const errorText = await res.text().catch(() => "");
    throw new Error(`API ${res.status}: ${errorText || res.statusText}`);
  }

  return res.json() as Promise<T>;
}

export const api = {
  health: () => apiFetch<{ status: string; uptime_secs: number; version?: string }>("/api/health"),
  metrics: () => apiFetch<MetricsData>("/api/metrics"),
  sessions: async () => {
    const res = await apiFetch<any>("/api/sessions");
    return (Array.isArray(res) ? res : res?.sessions || []) as SessionInfo[];
  },
  sessionHistory: async (id: string, limit = 100) => {
    const res = await apiFetch<any>(`/api/sessions/${encodeURIComponent(id)}/history?limit=${limit}`);
    return { messages: Array.isArray(res?.messages) ? res.messages : (Array.isArray(res) ? res : []) };
  },
  runs: async (limit = 50, offset = 0) => {
    const res = await apiFetch<any>(`/api/runs?limit=${limit}&offset=${offset}`);
    return {
      runs: Array.isArray(res?.runs) ? res.runs : (Array.isArray(res) ? res : []),
      total: typeof res?.total === "number" ? res.total : 0,
    };
  },
  costs: async (from?: string, to?: string, groupBy?: string) => {
    const params = new URLSearchParams();
    if (from) params.set("from", from);
    if (to) params.set("to", to);
    if (groupBy) params.set("group_by", groupBy);
    const res = await apiFetch<any>(`/api/costs?${params.toString()}`);
    return {
      costs: Array.isArray(res?.costs) ? res.costs : [],
      total_usd: typeof res?.total_usd === "number" ? res.total_usd : 0,
    };
  },
  audit: async (limit = 50, category?: string) => {
    const p = new URLSearchParams({ limit: String(limit) });
    if (category) p.set("category", category);
    const res = await apiFetch<any>(`/api/audit?${p.toString()}`);
    return (Array.isArray(res) ? res : res?.entries || []) as AuditEntry[];
  },
  auditStats: () => apiFetch<AuditStats>("/api/audit/stats"),
  safetyLessons: async () => {
    const res = await apiFetch<any>("/api/safety/lessons");
    return (Array.isArray(res) ? res : res?.lessons || []) as SafetyLesson[];
  },
  decisions: async (limit = 50) => {
    const res = await apiFetch<any>(`/api/decisions?limit=${limit}`);
    return (Array.isArray(res) ? res : res?.decisions || []) as DecisionEntry[];
  },
  failures: async () => {
    const res = await apiFetch<any>("/api/failures");
    return (Array.isArray(res) ? res : res?.failures || []) as FailureEntry[];
  },
  vikingList: async (path = "") => {
    const res = await apiFetch<any>(`/api/viking/list?path=${encodeURIComponent(path)}`);
    return (Array.isArray(res) ? res : []) as VikingNode[];
  },
  vikingRead: (path: string, level = "L0") =>
    apiFetch<VikingContent>(`/api/viking/read?path=${encodeURIComponent(path)}&level=${level}`),
  vikingSearch: async (q: string) => {
    const res = await apiFetch<any>(`/api/viking/search?q=${encodeURIComponent(q)}`);
    return (Array.isArray(res) ? res : []) as VikingContent[];
  },
  channels: async () => {
    const res = await apiFetch<any>("/api/channels");
    return (Array.isArray(res) ? res : res?.channels || []) as ChannelStatus[];
  },
  config: {
    get: () => apiFetch<Record<string, any>>("/api/config"),
    put: (cfg: Record<string, any>) =>
      apiFetch<{ ok: boolean }>("/api/config", { method: "PUT", body: JSON.stringify(cfg) }),
  },
  budget: {
    get: () => apiFetch<{ daily_limit_usd?: number; warning_threshold_pct?: number }>("/api/budget"),
    put: (body: any) => apiFetch<{ ok: boolean }>("/api/budget", { method: "PUT", body: JSON.stringify(body) }),
  },
  model: {
    get: () => apiFetch<ModelInfo>("/api/model"),
    put: (body: Partial<ModelInfo>) =>
      apiFetch<{ ok: boolean }>("/api/model", { method: "PUT", body: JSON.stringify(body) }),
    available: async (provider?: string) => {
      const url = provider ? `/api/models/available?provider=${encodeURIComponent(provider)}` : "/api/models/available";
      const res = await apiFetch<any>(url);
      return {
        provider: res?.provider || "",
        model_id: res?.model_id || "",
        models: Array.isArray(res?.models) ? res.models : [],
        all_providers: Array.isArray(res?.all_providers) ? res.all_providers : [],
        has_active_keys: Boolean(res?.has_active_keys),
        key_count: typeof res?.key_count === "number" ? res.key_count : 0,
      };
    },
  },
  keys: {
    list: () =>
      apiFetch<{
        providers: import("@/types/api").ProviderKeyInfo[];
        active_provider: string;
        active_model_id: string;
        active_keys_count: number;
      }>("/api/keys"),
    put: (data: {
      provider: string;
      api_key?: string;
      api_keys?: string[];
      base_url?: string;
      default_model?: string;
      activate?: boolean;
    }) =>
      apiFetch<{ ok: boolean; provider: string; is_active: boolean; key_count: number }>("/api/keys", {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    delete: (provider: string) =>
      apiFetch<{ ok: boolean; deleted: string }>(`/api/keys/${encodeURIComponent(provider)}`, {
        method: "DELETE",
      }),
  },
  integrations: {
    list: async () => {
      const res = await apiFetch<any>("/api/integrations");
      return (Array.isArray(res) ? res : res?.apps || []) as IntegrationItem[];
    },
    connect: (app: string, config: any) =>
      apiFetch<{ ok: boolean }>(`/api/integrations/${encodeURIComponent(app)}/connect`, {
        method: "POST",
        body: JSON.stringify(config),
      }),
    disconnect: (app: string) =>
      apiFetch<{ ok: boolean }>(`/api/integrations/${encodeURIComponent(app)}`, { method: "DELETE" }),
  },
  approvals: {
    list: async () => {
      const res = await apiFetch<any>("/api/approvals");
      return (Array.isArray(res) ? res : res?.approvals || []) as ApprovalRequest[];
    },
    approve: (id: string) =>
      apiFetch<{ ok: boolean }>(`/api/approvals/${encodeURIComponent(id)}/approve`, { method: "POST" }),
    deny: (id: string) =>
      apiFetch<{ ok: boolean }>(`/api/approvals/${encodeURIComponent(id)}/deny`, { method: "POST" }),
  },
  cron: {
    list: async () => {
      const res = await apiFetch<any>("/api/cron");
      return (Array.isArray(res) ? res : res?.jobs || []) as CronJob[];
    },
    add: (job: { name: string; expr: string; prompt: string }) =>
      apiFetch<{ ok: boolean }>("/api/cron", { method: "POST", body: JSON.stringify(job) }),
    delete: (name: string) =>
      apiFetch<{ ok: boolean }>(`/api/cron/${encodeURIComponent(name)}`, { method: "DELETE" }),
  },
  skills: async () => {
    const res = await apiFetch<any>("/api/skills");
    return { skills: Array.isArray(res?.skills) ? res.skills : [] };
  },
  heartbeatHistory: async () => {
    const res = await apiFetch<any>("/api/heartbeat/history");
    return (Array.isArray(res) ? res : res?.events || []) as any[];
  },
  goals: {
    run: (payload: { prompt: string; max_turns: number; judge_model?: string }) =>
      apiFetch<GoalRun>("/api/goals/run", { method: "POST", body: JSON.stringify(payload) }),
    history: async () => {
      const res = await apiFetch<any>("/api/goals/history");
      return (Array.isArray(res) ? res : res?.runs || []) as GoalRun[];
    },
  },
};
