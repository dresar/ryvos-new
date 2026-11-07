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
      error = e.message;
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
    if (!ts) return '-';
    try {
      const d = new Date(ts);
      return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    } catch { return '-'; }
  }
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Sessions</h2>
    <p class="text-[#778399] text-sm mt-1">Active conversation sessions and channel contexts</p>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading sessions...</p>
    </div>
  {:else if error}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm">Failed to load sessions: {error}</p>
    </div>
  {:else if sessionList.length === 0}
    <div class="card-soft p-12 text-center">
      <div class="w-12 h-12 rounded-2xl bg-[#E8EEFF] text-[#2448CB] flex items-center justify-center mx-auto mb-3">
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>
      </div>
      <h3 class="text-base font-semibold text-[#1B2943] mb-1">No active sessions</h3>
      <p class="text-xs text-[#778399] max-w-sm mx-auto">Conversations opened in Web UI or adapters will appear here.</p>
    </div>
  {:else}
    <div class="card-soft overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                Session ID
              </th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                Channel
              </th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                Last Active
              </th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                Runs
              </th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[#DCE4F3]">
            {#each sessionList as session}
              <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                <td class="px-4 py-3 font-mono text-xs text-[#1B2943]">
                  {truncate(getSid(session), 40)}
                </td>
                <td class="px-4 py-3 text-xs text-[#64748B]">
                  <span class="px-2 py-0.5 rounded-full bg-[#E8EEFF] text-[#2448CB] font-medium">
                    {session.channel || 'webui'}
                  </span>
                </td>
                <td class="px-4 py-3 text-xs text-[#778399] font-mono tabular-nums">
                  {formatTime(session.last_active)}
                </td>
                <td class="px-4 py-3 text-xs text-[#1B2943] tabular-nums font-semibold">
                  {session.total_runs ?? '-'}
                </td>
                <td class="px-4 py-3">
                  <a
                    href="#/chat/{encodeURIComponent(getSid(session))}"
                    class="btn-surface inline-flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-[#2448CB] hover:bg-[#E8EEFF]"
                  >
                    <span>Open Chat</span>
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
