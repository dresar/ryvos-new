<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let sessionList = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      const data = await apiFetch('/api/sessions');
      sessionList = data.sessions || [];
    } catch (e) {
      error = e.message || 'Failed to load sessions';
    } finally {
      loading = false;
    }
  });

  function getSid(session) {
    return typeof session === 'string' ? session : (session.id || String(session));
  }

  function truncate(s, max) {
    if (!s || typeof s !== 'string') return String(s || '');
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  function formatTime(ts) {
    if (!ts) return 'Never';
    try {
      const d = new Date(ts);
      return d.toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return String(ts);
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Conversation Sessions</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Active conversation sessions, execution channels, and runtime session metadata</p>
  </div>

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading registered sessions...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Session Indexing Error</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else if sessionList.length === 0}
    <div class="neu-card p-12 text-center">
      <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Active Sessions</p>
      <p class="text-xs text-[var(--color-text-muted)]">Conversations initiated via Web UI or external gateways will be listed here.</p>
    </div>
  {:else}
    <div class="neu-card p-0 overflow-hidden">
      <div class="table-scroll">
        <table class="w-full text-left text-sm">
          <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
            <tr>
              <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                Session Identifier
              </th>
              <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                Channel
              </th>
              <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                Last Activity
              </th>
              <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                Run Count
              </th>
              <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                Action
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[var(--color-border)]">
            {#each sessionList as session}
              {@const sid = getSid(session)}
              <tr class="hover:bg-[var(--color-bg)] transition-colors">
                <td class="px-4 py-3 font-mono text-xs text-[var(--color-text-main)] whitespace-nowrap">
                  {truncate(sid, 36)}
                </td>
                <td class="px-4 py-3 text-xs whitespace-nowrap">
                  <span class="inline-flex px-2 py-0.5 rounded-full font-semibold badge-info">
                    {session.channel || 'webui'}
                  </span>
                </td>
                <td class="px-4 py-3 text-xs text-[var(--color-text-muted)] font-mono tabular-nums whitespace-nowrap">
                  {formatTime(session.last_active)}
                </td>
                <td class="px-4 py-3 text-xs text-[var(--color-text-main)] font-mono tabular-nums whitespace-nowrap">
                  {session.total_runs ?? 1}
                </td>
                <td class="px-4 py-3 whitespace-nowrap">
                  <a
                    href={'#/chat/' + encodeURIComponent(sid)}
                    class="btn-soft inline-flex items-center gap-1.5 min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl text-[var(--color-primary)]"
                  >
                    <span>Resume Chat</span>
                    <span aria-hidden="true">&rarr;</span>
                  </a>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>