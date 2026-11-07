<script>
  import { onMount, onDestroy } from 'svelte';
  import { apiFetch } from '../api.js';
  import { activityFeed } from '../ws.js';
  import ActivityFeed from '../components/ActivityFeed.svelte';

  let loading = $state(true);
  let error = $state('');
  let uptimeSecs = $state(0);
  let activeSessions = $state(0);
  let totalEntries = $state(0);
  let toolBreakdown = $state([]);
  let heartbeatSessions = $state(0);
  let vikingEntries = $state(0);
  let channelCount = $state(0);
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
    if (n >= 1000) return (n / 1000).toFixed(1) + 'k';
    return String(n);
  }

  let totalToolCalls = $derived(toolBreakdown.reduce((sum, t) => sum + (t.count || 0), 0));
  let maxToolCount = $derived(toolBreakdown.length > 0 ? toolBreakdown[0].count : 1);

  onMount(async () => {
    unsubActivity = activityFeed.subscribe(feed => {
      guardianAlerts = feed.filter(item =>
        item.kind === 'guardian_stall' ||
        item.kind === 'guardian_doom_loop' ||
        item.kind === 'guardian_budget_alert'
      ).slice(0, 5);
    });

    try {
      const [metrics, auditStats, channels, health] = await Promise.all([
        apiFetch('/api/metrics').catch(() => null),
        apiFetch('/api/audit/stats').catch(() => null),
        apiFetch('/api/channels').catch(() => []),
        apiFetch('/api/health').catch(() => null),
      ]);

      if (metrics) {
        uptimeSecs = metrics.uptime_secs || 0;
        activeSessions = metrics.active_sessions || 0;
      }

      if (auditStats) {
        totalEntries = auditStats.total_entries || 0;
        toolBreakdown = auditStats.tool_breakdown || [];
        heartbeatSessions = auditStats.heartbeat_sessions || 0;
        vikingEntries = auditStats.viking_entries || 0;
      }

      if (Array.isArray(channels)) {
        channelCount = channels.length;
      } else if (channels && typeof channels === 'object') {
        channelCount = Object.keys(channels).length;
      }

      if (health) {
        version = health.version || 'unknown';
        status = health.status || 'unknown';
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
    <h1 class="text-2xl md:text-3xl font-heading font-bold text-[#1B2943]">Dashboard</h1>
    <p class="text-sm text-[#778399] mt-1">Instance metrics, active sessions, and runtime health</p>
  </div>

  {#if guardianAlerts.length > 0}
    <div class="border border-[#D94B5B] bg-[#D94B5B]/5 rounded-2xl p-4 shadow-soft-sm">
      <div class="flex items-center gap-2 mb-2">
        <svg class="w-5 h-5 text-[#D94B5B]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        <span class="label text-[#D94B5B]">Guardian Alerts</span>
      </div>
      {#each guardianAlerts as alert}
        <div class="flex items-center gap-2 text-xs md:text-sm text-[#D94B5B] py-1 font-mono">
          <span class="text-xs text-[#778399] tabular-nums">{alert.time}</span>
          <span>{alert.detail}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if loading}
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3 md:gap-4">
      {#each Array(5) as _}
        <div class="bg-white border border-[#DCE4F3] rounded-2xl p-4 shadow-soft animate-pulse">
          <div class="h-3 bg-[#EAF0FC] rounded w-20 mb-3"></div>
          <div class="h-7 bg-[#EAF0FC] rounded w-14"></div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3 md:gap-4">
      <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-4 md:p-5 transition-all duration-200 hover:shadow-soft-float">
        <div class="flex items-center gap-2 text-[#778399] mb-1.5">
          <div class="p-1.5 rounded-lg bg-[#E8EEFF] text-[#2448CB]">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
            </svg>
          </div>
          <span class="label text-[10px]">Heartbeats</span>
        </div>
        <p class="text-2xl font-heading font-bold text-[#1B2943] tabular-nums">{formatNumber(heartbeatSessions)}</p>
      </div>

      <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-4 md:p-5 transition-all duration-200 hover:shadow-soft-float">
        <div class="flex items-center gap-2 text-[#778399] mb-1.5">
          <div class="p-1.5 rounded-lg bg-[#E8EEFF] text-[#2448CB]">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/>
            </svg>
          </div>
          <span class="label text-[10px]">Sessions</span>
        </div>
        <p class="text-2xl font-heading font-bold text-[#1B2943] tabular-nums">{activeSessions}</p>
      </div>

      <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-4 md:p-5 transition-all duration-200 hover:shadow-soft-float">
        <div class="flex items-center gap-2 text-[#778399] mb-1.5">
          <div class="p-1.5 rounded-lg bg-[#E8EEFF] text-[#2448CB]">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/>
            </svg>
          </div>
          <span class="label text-[10px]">Tool Calls</span>
        </div>
        <p class="text-2xl font-heading font-bold text-[#1B2943] tabular-nums">{formatNumber(totalToolCalls)}</p>
      </div>

      <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-4 md:p-5 transition-all duration-200 hover:shadow-soft-float">
        <div class="flex items-center gap-2 text-[#778399] mb-1.5">
          <div class="p-1.5 rounded-lg bg-[#E8EEFF] text-[#2448CB]">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/>
            </svg>
          </div>
          <span class="label text-[10px]">Memory Context</span>
        </div>
        <p class="text-2xl font-heading font-bold text-[#1B2943] tabular-nums">{formatNumber(vikingEntries)}</p>
      </div>

      <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-4 md:p-5 transition-all duration-200 hover:shadow-soft-float col-span-2 md:col-span-1">
        <div class="flex items-center gap-2 text-[#778399] mb-1.5">
          <div class="p-1.5 rounded-lg bg-[#E8EEFF] text-[#2448CB]">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
            </svg>
          </div>
          <span class="label text-[10px]">Uptime</span>
        </div>
        <p class="text-2xl font-heading font-bold text-[#1B2943] tabular-nums">{formatUptime(uptimeSecs)}</p>
      </div>
    </div>
  {/if}

  {#if !loading && toolBreakdown.length > 0}
    <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-5">
      <h2 class="label mb-4 text-[#1B2943]">Tool Usage</h2>
      <div class="space-y-3">
        {#each toolBreakdown.slice(0, 8) as item}
          <div class="flex items-center gap-3">
            <span class="text-xs font-mono w-36 text-[#64748B] truncate">{item.tool}</span>
            <div class="flex-1 bg-[#EAF0FC] h-4 rounded-full overflow-hidden">
              <div class="bg-[#2448CB] h-full rounded-full transition-all duration-500" style="width: {(item.count / maxToolCount * 100)}%"></div>
            </div>
            <span class="text-xs font-mono tabular-nums text-[#778399] w-12 text-right">{item.count}</span>
          </div>
        {/each}
      </div>
    </div>
  {:else if !loading}
    <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-5">
      <h2 class="label mb-4 text-[#1B2943]">Tool Usage</h2>
      <p class="text-sm text-[#778399] py-4 text-center">No tool calls recorded yet. Tools will appear here as agents execute actions.</p>
    </div>
  {/if}

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
    <ActivityFeed />

    <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-5">
      <h2 class="label mb-4 text-[#1B2943]">System Overview</h2>
      {#if loading}
        <div class="space-y-4 animate-pulse">
          <div>
            <div class="h-3 bg-[#EAF0FC] rounded w-16 mb-2"></div>
            <div class="h-6 bg-[#EAF0FC] rounded w-24"></div>
          </div>
          <div>
            <div class="h-3 bg-[#EAF0FC] rounded w-16 mb-2"></div>
            <div class="h-6 bg-[#EAF0FC] rounded w-20"></div>
          </div>
        </div>
      {:else}
        <div class="grid grid-cols-2 gap-4">
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Version</span>
            <p class="text-base font-bold font-heading text-[#1B2943] mt-0.5">{version || 'unknown'}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Status</span>
            <div class="flex items-center gap-2 mt-0.5">
              {#if status === 'ok' || status === 'healthy'}
                <span class="w-2.5 h-2.5 rounded-full bg-[#219B70] inline-block animate-pulse-glow"></span>
                <p class="text-base font-bold font-heading text-[#219B70] uppercase">{status}</p>
              {:else}
                <p class="text-base font-bold font-heading text-[#1B2943]">{status || 'unknown'}</p>
              {/if}
            </div>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Channels Active</span>
            <p class="text-base font-bold font-heading text-[#1B2943] tabular-nums mt-0.5">{channelCount}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Audit Log Entries</span>
            <p class="text-base font-bold font-heading text-[#1B2943] tabular-nums mt-0.5">{formatNumber(totalEntries)}</p>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div>
    <h2 class="label mb-3 text-[#1B2943]">Quick Actions</h2>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
      <a href="#/chat" class="btn-surface flex items-center gap-3 p-4 text-sm text-[#1B2943] font-medium hover:border-[#2448CB]">
        <div class="p-2 rounded-xl bg-[#E8EEFF] text-[#2448CB]">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
          </svg>
        </div>
        <span>Chat with agent</span>
        <span class="ml-auto text-[#778399]">&rarr;</span>
      </a>
      <a href="#/viking" class="btn-surface flex items-center gap-3 p-4 text-sm text-[#1B2943] font-medium hover:border-[#2448CB]">
        <div class="p-2 rounded-xl bg-[#E8EEFF] text-[#2448CB]">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/>
          </svg>
        </div>
        <span>Browse memory</span>
        <span class="ml-auto text-[#778399]">&rarr;</span>
      </a>
      <a href="#/audit" class="btn-surface flex items-center gap-3 p-4 text-sm text-[#1B2943] font-medium hover:border-[#2448CB]">
        <div class="p-2 rounded-xl bg-[#E8EEFF] text-[#2448CB]">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          </svg>
        </div>
        <span>View audit trail</span>
        <span class="ml-auto text-[#778399]">&rarr;</span>
      </a>
      <a href="#/channels" class="btn-surface flex items-center gap-3 p-4 text-sm text-[#1B2943] font-medium hover:border-[#2448CB]">
        <div class="p-2 rounded-xl bg-[#E8EEFF] text-[#2448CB]">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 00-2-2h-4a2 2 0 00-2 2v16"/>
          </svg>
        </div>
        <span>Check channels</span>
        <span class="ml-auto text-[#778399]">&rarr;</span>
      </a>
    </div>
  </div>

  {#if error}
    <div class="border border-[#D94B5B] bg-white rounded-2xl p-4 shadow-soft-sm">
      <p class="text-sm text-[#D94B5B]">Failed to load some dashboard data: {error}</p>
    </div>
  {/if}
</div>
