<script>
  import { onMount, onDestroy } from 'svelte';
  import { apiFetch } from '../api.js';
  import { activityFeed } from '../ws.js';
  import ActivityFeed from '../components/ActivityFeed.svelte';

  let loading = $state(true);
  let error = $state('');
  let uptimeSecs = $state(0);
  let activeSessions = $state(0);
  let totalRuns = $state(0);
  let totalTokens = $state(0);
  let toolBreakdown = $state([]);
  let vikingEntries = $state(0);
  let version = $state('');
  let status = $state('');
  let guardianAlerts = $state([]);

  let unsubActivity = () => {};

  function formatUptime(secs) {
    if (!secs && secs !== 0) return '-';
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    if (d > 0) return `${d}d ${h}h`;
    return `${h}h ${m}m`;
  }

  function formatNumber(n) {
    if (!n && n !== 0) return '0';
    if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
    if (n >= 1000) return (n / 1000).toFixed(1) + 'k';
    return String(n);
  }

  let totalToolCalls = $derived(toolBreakdown.reduce((sum, t) => sum + (t.count || 0), 0));
  let maxToolCount = $derived(toolBreakdown.length > 0 ? Math.max(...toolBreakdown.map(t => t.count || 0), 1) : 1);

  onMount(async () => {
    unsubActivity = activityFeed.subscribe(feed => {
      guardianAlerts = feed.filter(item =>
        item.kind === 'guardian_stall' ||
        item.kind === 'guardian_doom_loop' ||
        item.kind === 'guardian_budget_alert'
      ).slice(0, 5);
    });

    try {
      const [metrics, auditStats, health] = await Promise.all([
        apiFetch('/api/metrics').catch(() => null),
        apiFetch('/api/audit/stats').catch(() => null),
        apiFetch('/api/health').catch(() => null),
      ]);

      if (metrics) {
        uptimeSecs = metrics.uptime_secs || 0;
        activeSessions = metrics.active_sessions || 0;
        totalRuns = metrics.total_runs || 0;
        totalTokens = metrics.total_tokens || 0;
      }

      if (auditStats) {
        toolBreakdown = auditStats.tool_breakdown || [];
        vikingEntries = auditStats.viking_entries || 0;
      }

      if (health) {
        version = health.version || '0.9.0';
        status = health.status || 'ok';
      }
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    unsubActivity();
  });
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-xl sm:text-2xl font-bold tracking-tight text-[var(--color-text)]">
      Runtime Operations & Telemetry
    </h2>
    <p class="text-xs sm:text-sm text-[var(--color-text-secondary)] mt-1">
      Autonomous agent health status, execution statistics, and watchdog monitoring
    </p>
  </div>

  {#if loading}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
      {#each [1, 2, 3, 4, 5] as _}
        <div class="h-28 rounded-2xl skeleton"></div>
      {/each}
    </div>
    <div class="h-64 rounded-3xl skeleton"></div>
  {:else if error}
    <div class="neu-card p-8 text-center border-l-4 border-l-[var(--color-danger)]">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Telemetry Unavailable</h3>
      <p class="text-xs text-[var(--color-text-secondary)]">{error}</p>
    </div>
  {:else}
    {#if guardianAlerts.length > 0}
      <div class="neu-card border border-[var(--color-warning)] bg-[var(--color-warning)]/10 rounded-2xl p-4 shadow-sm">
        <div class="flex items-center gap-2 mb-2">
          <svg class="w-5 h-5 text-[var(--color-warning)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
          <span class="text-xs font-bold uppercase tracking-wider text-[var(--color-warning)]">
            Guardian Watchdog Alerts
          </span>
        </div>
        <div class="space-y-1">
          {#each guardianAlerts as alert}
            <div class="text-xs text-[var(--color-text)] flex items-center justify-between">
              <span>{alert.detail}</span>
              <span class="font-mono text-[11px] text-[var(--color-text-secondary)]">{alert.time}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <div class="neu-focal p-3 sm:p-5 flex flex-col md:flex-row md:items-center justify-between gap-3 sm:gap-6">
      <div class="space-y-1.5 sm:space-y-2">
        <div class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-bold bg-[var(--color-primary-soft)] text-[var(--color-primary)]">
          <span class="w-2 h-2 rounded-full bg-[var(--color-success)]"></span>
          Cluster Node: eka-12 (Production Ready)
        </div>
        <h3 class="text-base sm:text-2xl font-bold text-[var(--color-text)] tracking-tight">
          System Status: {status === 'ok' ? 'Nominal' : status}
        </h3>
        <p class="text-[11px] sm:text-sm text-[var(--color-text-secondary)] max-w-xl">
          Ryvos v{version} runtime running with rootless Bubblewrap sandbox, lock-free SSE streaming, and persistent Viking SQLite memory.
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2 sm:gap-3 shrink-0">
        <a href="#/chat" class="btn-primary text-xs px-3 py-1.5 min-h-[36px]">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
          </svg>
          <span>Start Agent Session</span>
        </a>
        <a href="#/goals" class="btn-soft text-xs px-3 py-1.5 min-h-[36px]">
          <svg class="w-4 h-4 text-[var(--color-primary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>
          </svg>
          <span>Launch Goal Run</span>
        </a>
      </div>
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-2 lg:grid-cols-5 gap-2 sm:gap-3">
      <div class="neu-card p-2.5 sm:p-4">
        <p class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Active Sessions</p>
        <p class="mt-1 sm:mt-2 text-lg sm:text-2xl font-bold text-[var(--color-text)] tabular-nums">{activeSessions}</p>
        <p class="mt-0.5 sm:mt-1 text-[10px] sm:text-xs text-[var(--color-text-secondary)]">WebUI & WebSocket</p>
      </div>

      <div class="neu-card p-2.5 sm:p-4">
        <p class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Total Runs</p>
        <p class="mt-1 sm:mt-2 text-lg sm:text-2xl font-bold text-[var(--color-text)] tabular-nums">{formatNumber(totalRuns)}</p>
        <p class="mt-0.5 sm:mt-1 text-[10px] sm:text-xs text-[var(--color-text-secondary)]">Executed cycles</p>
      </div>

      <div class="neu-card p-2.5 sm:p-4">
        <p class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Total Tokens</p>
        <p class="mt-1 sm:mt-2 text-lg sm:text-2xl font-bold text-[var(--color-text)] tabular-nums">{formatNumber(totalTokens)}</p>
        <p class="mt-0.5 sm:mt-1 text-[10px] sm:text-xs text-[var(--color-text-secondary)]">Input & output</p>
      </div>

      <div class="neu-card p-2.5 sm:p-4">
        <p class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Viking Memories</p>
        <p class="mt-1 sm:mt-2 text-lg sm:text-2xl font-bold text-[var(--color-text)] tabular-nums">{formatNumber(vikingEntries)}</p>
        <p class="mt-0.5 sm:mt-1 text-[10px] sm:text-xs text-[var(--color-text-secondary)]">L0/L1/L2 database</p>
      </div>

      <div class="neu-card p-2.5 sm:p-4 col-span-2 sm:col-span-1">
        <p class="text-[10px] sm:text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Node Uptime</p>
        <p class="mt-1 sm:mt-2 text-lg sm:text-2xl font-bold text-[var(--color-text)] tabular-nums">{formatUptime(uptimeSecs)}</p>
        <p class="mt-0.5 sm:mt-1 text-[10px] sm:text-xs text-[var(--color-text-secondary)]">Continuous runtime</p>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="neu-card p-5 sm:p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-sm font-bold text-[var(--color-text)] m-0">Tool Execution Frequency</h3>
          <span class="text-xs font-semibold text-[var(--color-text-secondary)] tabular-nums">
            {formatNumber(totalToolCalls)} total calls
          </span>
        </div>

        {#if toolBreakdown.length === 0}
          <div class="py-12 text-center text-xs text-[var(--color-text-secondary)]">
            No tool executions recorded yet in current audit log.
          </div>
        {:else}
          <div class="space-y-3">
            {#each toolBreakdown.slice(0, 6) as tool}
              <div>
                <div class="flex items-center justify-between text-xs mb-1">
                  <span class="font-mono font-semibold text-[var(--color-primary)]">{tool.name}</span>
                  <span class="font-mono text-[var(--color-text-secondary)] tabular-nums">{tool.count}</span>
                </div>
                <div class="neu-inset h-2 rounded-full overflow-hidden p-0.5">
                  <div
                    class="h-full rounded-full transition-all duration-300"
                    style="width: {Math.max((tool.count / maxToolCount) * 100, 4)}%; background: var(--gradient-primary);"
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <ActivityFeed />
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
      <a href="#/chat" class="neu-card p-4 hover:border-[var(--color-primary)] transition-all">
        <div class="w-8 h-8 rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center mb-2">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        </div>
        <span class="text-sm font-bold text-[var(--color-text)] block">Agent Chat</span>
        <span class="text-xs text-[var(--color-text-secondary)]">Prompt & tools</span>
      </a>

      <a href="#/viking" class="neu-card p-4 hover:border-[var(--color-primary)] transition-all">
        <div class="w-8 h-8 rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center mb-2">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
        </div>
        <span class="text-sm font-bold text-[var(--color-text)] block">Viking Context</span>
        <span class="text-xs text-[var(--color-text-secondary)]">Memory browser</span>
      </a>

      <a href="#/audit" class="neu-card p-4 hover:border-[var(--color-primary)] transition-all">
        <div class="w-8 h-8 rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center mb-2">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
        </div>
        <span class="text-sm font-bold text-[var(--color-text)] block">Audit Trail</span>
        <span class="text-xs text-[var(--color-text-secondary)]">Lessons & safety</span>
      </a>

      <a href="#/channels" class="neu-card p-4 hover:border-[var(--color-primary)] transition-all">
        <div class="w-8 h-8 rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center mb-2">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 010 8.49m-8.48-.01a6 6 0 010-8.49m11.31-2.82a10 10 0 010 14.14m-14.14 0a10 10 0 010-14.14"/></svg>
        </div>
        <span class="text-sm font-bold text-[var(--color-text)] block">Channels</span>
        <span class="text-xs text-[var(--color-text-secondary)]">Adapters</span>
      </a>
    </div>
  {/if}
</div>
