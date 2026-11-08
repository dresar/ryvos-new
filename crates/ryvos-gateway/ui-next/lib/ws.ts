import type { ActivityEvent, ModelInfo } from "@/types/api";

type ConnectionState = "connected" | "disconnected" | "error";

type Listener<T> = (val: T) => void;

class WsClient {
  private ws: WebSocket | null = null;
  private reconnectTimer: any = null;
  private retryCount = 0;
  private readonly maxRetryDelay = 30000;
  private pendingRequests = new Map<
    string,
    { resolve: (val: any) => void; reject: (err: Error) => void; timer: any }
  >();
  private nextRequestId = 1;

  public status: ConnectionState = "disconnected";
  public streamingText = "";
  public activityFeed: ActivityEvent[] = [];
  public modelInfo: ModelInfo = {
    provider: "gemini",
    model_id: "gemini-3.6-flash",
    key_count: 0,
  };

  private statusListeners = new Set<Listener<ConnectionState>>();
  private streamListeners = new Set<Listener<string>>();
  private feedListeners = new Set<Listener<ActivityEvent[]>>();
  private modelListeners = new Set<Listener<ModelInfo>>();
  private rawEventListeners = new Set<(event: any, sessionId?: string) => void>();

  public onStatus(cb: Listener<ConnectionState>): () => void {
    this.statusListeners.add(cb);
    cb(this.status);
    return () => this.statusListeners.delete(cb);
  }

  public onStream(cb: Listener<string>): () => void {
    this.streamListeners.add(cb);
    cb(this.streamingText);
    return () => this.streamListeners.delete(cb);
  }

  public onFeed(cb: Listener<ActivityEvent[]>): () => void {
    this.feedListeners.add(cb);
    cb(this.activityFeed);
    return () => this.feedListeners.delete(cb);
  }

  public onModel(cb: Listener<ModelInfo>): () => void {
    this.modelListeners.add(cb);
    cb(this.modelInfo);
    return () => this.modelListeners.delete(cb);
  }

  public onRawEvent(cb: (event: any, sessionId?: string) => void): () => void {
    this.rawEventListeners.add(cb);
    return () => this.rawEventListeners.delete(cb);
  }

  private setStatus(s: ConnectionState) {
    this.status = s;
    this.statusListeners.forEach((fn) => fn(s));
  }

  private appendStream(delta: string) {
    this.streamingText += delta;
    this.streamListeners.forEach((fn) => fn(this.streamingText));
  }

  public resetStreamingText() {
    this.streamingText = "";
    this.streamListeners.forEach((fn) => fn(this.streamingText));
  }

  public connect(apiKey?: string) {
    if (typeof window === "undefined") return;
    if (this.ws) {
      try {
        this.ws.close();
      } catch {}
    }
    clearTimeout(this.reconnectTimer);

    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    const host = window.location.host;
    const url = `${proto}//${host}/ws${apiKey ? `?token=${encodeURIComponent(apiKey)}` : ""}`;

    try {
      this.ws = new WebSocket(url);
    } catch {
      this.setStatus("error");
      return;
    }

    this.ws.onopen = () => {
      this.setStatus("connected");
      this.retryCount = 0;
      this.sendWs("model.get")
        .then((info) => {
          if (info && info.provider) {
            this.modelInfo = info;
            this.modelListeners.forEach((fn) => fn(info));
          }
        })
        .catch(() => {});
    };

    this.ws.onclose = () => {
      this.setStatus("disconnected");
      this.pendingRequests.forEach((req) => {
        clearTimeout(req.timer);
        req.reject(new Error("WebSocket disconnected"));
      });
      this.pendingRequests.clear();

      const delay = Math.min(1000 * Math.pow(2, this.retryCount), this.maxRetryDelay);
      this.retryCount++;
      this.reconnectTimer = setTimeout(() => this.connect(apiKey), delay);
    };

    this.ws.onerror = () => {
      this.setStatus("error");
    };

    this.ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data);
        if (msg.type === "response" && msg.id) {
          const idStr = String(msg.id);
          const req = this.pendingRequests.get(idStr);
          if (req) {
            clearTimeout(req.timer);
            this.pendingRequests.delete(idStr);
            if (msg.error) {
              req.reject(new Error(msg.error.message || "Server error"));
            } else if (msg.result && msg.result.error) {
              req.reject(new Error(msg.result.error));
            } else {
              req.resolve(msg.result);
            }
          }
        } else if (msg.type === "event") {
          const event = msg.event || {};
          const sessionId = msg.session_id;

          if (event.kind === "text_delta") {
            this.appendStream(event.text || "");
          } else {
            const entry: ActivityEvent = {
              time: new Date().toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
              }),
              kind: event.kind,
              text: event.text || event.tool || "",
              session: sessionId ? sessionId.substring(0, 10) : "",
              detail: this.formatEventDetail(event),
              data: event.data,
            };
            this.activityFeed = [entry, ...this.activityFeed].slice(0, 100);
            this.feedListeners.forEach((fn) => fn(this.activityFeed));
          }

          this.rawEventListeners.forEach((fn) => fn(event, sessionId));
        }
      } catch {}
    };
  }

  private formatEventDetail(evt: any): string {
    switch (evt.kind) {
      case "run_started":
        return "Run started";
      case "run_complete":
        return `Completed in ${evt.data?.total_turns || "?"} turns`;
      case "run_error":
        return `Error: ${evt.data?.error || "?"}`;
      case "tool_start":
        return `Tool: ${evt.tool || "?"}`;
      case "tool_end":
        return `Tool done: ${evt.tool || "?"}`;
      case "usage_update":
        return `+${evt.data?.input_tokens || 0} in / +${evt.data?.output_tokens || 0} out`;
      case "budget_warning":
        return `Budget at ${evt.data?.utilization_pct || "?"}%`;
      case "budget_exceeded":
        return "Budget exceeded!";
      case "heartbeat_fired":
        return "Heartbeat fired";
      case "heartbeat_ok":
        return `Heartbeat OK (${evt.data?.response_chars || 0} chars)`;
      case "heartbeat_alert":
        return `Heartbeat ALERT: ${(evt.data?.message || "").slice(0, 60)}`;
      case "cron_fired":
        return `Cron: ${evt.data?.job_name || "?"}`;
      case "cron_complete":
        return `Cron done: ${evt.data?.job_name || "?"}`;
      case "guardian_stall":
        return "Guardian: stall detected";
      case "guardian_doom_loop":
        return "Guardian: doom loop detected";
      case "guardian_budget_alert":
        return "Guardian: budget alert";
      case "approval_requested":
        return `Approval needed: ${evt.data?.tool_name || "?"}`;
      case "tool_blocked":
        return `Blocked: ${evt.tool || "?"}`;
      default:
        return evt.kind || "";
    }
  }

  public disconnect() {
    clearTimeout(this.reconnectTimer);
    if (this.ws) {
      try {
        this.ws.close();
      } catch {}
    }
    this.ws = null;
    this.setStatus("disconnected");
  }

  public sendWs(method: string, params: Record<string, any> = {}): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error("WebSocket is not connected"));
        return;
      }
      const id = String(this.nextRequestId++);
      const timer = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timed out: ${method}`));
      }, 120000);

      this.pendingRequests.set(id, { resolve, reject, timer });
      this.ws.send(
        JSON.stringify({
          type: "request",
          id,
          method,
          params,
        })
      );
    });
  }
}

export const wsClient = new WsClient();
