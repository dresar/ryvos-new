export interface SessionInfo {
  session_id: string;
  channel: string;
  turn_count: number;
  last_active: string;
  created_at: string;
}

export interface ChatMessage {
  id?: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp?: string;
  tool_calls?: ToolCallItem[];
  think?: string;
}

export interface ToolCallItem {
  id: string;
  name: string;
  input: string;
  output?: string;
  status: "running" | "completed" | "failed";
  elapsed_ms?: number;
}

export interface ApprovalRequest {
  id: string;
  session_id: string;
  tool_name: string;
  input: string;
  created_at: string;
}

export interface MetricsData {
  uptime_secs: number;
  total_sessions: number;
  active_sessions: number;
  total_runs: number;
  total_turns: number;
  total_input_tokens: number;
  total_output_tokens: number;
  active_runs: number;
  rss_bytes: number;
  cgroup_limit_bytes: number;
  cpu_percent: number;
  total_cost_usd: number;
  health_status: string;
}

export interface RunRecord {
  run_id: string;
  session_id: string;
  status: "running" | "completed" | "failed" | "cancelled";
  started_at: string;
  ended_at?: string;
  duration_ms?: number;
  turns: number;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  error?: string;
}

export interface CostRecord {
  id: string;
  session_id: string;
  model: string;
  provider: string;
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  timestamp: string;
}

export interface AuditEntry {
  id: string;
  timestamp: string;
  session_id?: string;
  action: string;
  category: "security" | "ssrf" | "tool_block" | "decision" | "failure";
  detail: string;
  metadata?: Record<string, unknown>;
}

export interface AuditStats {
  total_events: number;
  ssrf_blocks: number;
  tool_blocks: number;
  warnings: number;
  decisions: number;
  failures: number;
}

export interface SafetyLesson {
  id: string;
  pattern: string;
  action: string;
  reason: string;
  learned_at: string;
}

export interface DecisionEntry {
  id: string;
  timestamp: string;
  session_id: string;
  choice: string;
  alternatives: string[];
  justification: string;
}

export interface FailureEntry {
  id: string;
  timestamp: string;
  error: string;
  root_cause: string;
  mitigation: string;
  recovered: boolean;
}

export interface VikingNode {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number;
  children?: VikingNode[];
}

export interface VikingContent {
  path: string;
  level: "L0" | "L1" | "L2";
  content: string;
  summary?: string;
}

export interface ChannelStatus {
  name: string;
  enabled: boolean;
  status: "connected" | "disconnected" | "error" | "not_configured";
  info?: string;
}

export interface IntegrationItem {
  id: string;
  name: string;
  provider?: string;
  type?: string;
  status?: "connected" | "disconnected" | "error";
  connected?: boolean;
  configured?: boolean;
  actions?: number;
  icon?: string;
  description?: string;
  config?: Record<string, unknown>;
}

export interface CronJob {
  name: string;
  expr: string;
  prompt: string;
  last_run?: string;
  next_run?: string;
  status?: string;
}

export interface GoalRun {
  goal_id: string;
  prompt: string;
  max_turns: number;
  judge_model?: string;
  status: "running" | "completed" | "failed";
  verdict?: "SUCCESS" | "FAILURE";
  score?: number;
  critique?: string;
  trace?: string[];
  created_at: string;
}

export interface ModelInfo {
  provider: string;
  model_id: string;
  key_count: number;
  api_keys?: string[];
}

export interface ActivityEvent {
  time: string;
  kind: string;
  text: string;
  session: string;
  detail: string;
  data?: any;
}

export interface ModelPreset {
  id: string;
  name: string;
}

export interface ProviderKeyInfo {
  id: string;
  name: string;
  has_key: boolean;
  key_count: number;
  masked_keys: string[];
  base_url?: string | null;
  is_active: boolean;
  default_model?: string;
  models: ModelPreset[];
}
