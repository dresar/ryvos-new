<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let entries = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filterTool = $state('');

  async function loadAudit() {
    loading = true;
    error = '';
    try {
      let url = '/api/audit?limit=50';
      if (filterTool) url += `&tool=${encodeURIComponent(filterTool)}`;
      const data = await apiFetch(url);
      entries = data.entries || data.events || data.audit || [];
    } catch (e) {
      error = e.message;
      entries = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => loadAudit());

  function truncate(s, max) {
    if (!s) return '';
    if (typeof s === 'object') s = JSON.stringify(s);
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  function formatTime(isoStr) {
    if (!isoStr) return '-';
    return new Date(isoStr).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  function handleFilter() {
    loadAudit();
  }
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Audit Trail</h2>
    <p class="text-[#778399] text-sm mt-1">Immutable security log of tool executions and system events</p>
  </div>

  <div class="card-soft p-4 mb-6 flex items-center gap-3 flex-wrap">
    <label class="flex items-center gap-2 text-xs text-[#64748B] font-medium">
      Filter by tool:
      <input
        type="text"
        bind:value={filterTool}
        placeholder="e.g. bash, read_file"
        class="input-soft px-3 py-1.5 text-xs text-[#1B2943] w-48 font-mono"
      />
    </label>
    <button
      type="button"
      onclick={handleFilter}
      class="btn-primary px-4 py-1.5 text-xs font-semibold"
    >
      Apply
    </button>
    {#if filterTool}
      <button
        type="button"
        onclick={() => { filterTool = ''; loadAudit(); }}
        class="btn-surface px-3 py-1.5 text-xs font-semibold text-[#778399]"
      >
        Clear
      </button>
    {/if}
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading audit entries...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Audit trail not available: {error}</p>
    </div>
  {:else if entries.length === 0}
    <div class="card-soft p-12 text-center">
      <div class="w-12 h-12 rounded-2xl bg-[#E8EEFF] text-[#2448CB] flex items-center justify-center mx-auto mb-3">
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      </div>
      <h3 class="text-base font-semibold text-[#1B2943] mb-1">No audit entries found</h3>
      <p class="text-xs text-[#778399] max-w-sm mx-auto">Tool execution entries and sandbox calls will be recorded here.</p>
    </div>
  {:else}
    <div class="card-soft overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Time</th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Tool</th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Input</th>
              <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Outcome</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[#DCE4F3]">
            {#each entries as entry}
              <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                <td class="px-4 py-3 font-mono text-xs text-[#778399] tabular-nums">{formatTime(entry.timestamp || entry.time)}</td>
                <td class="px-4 py-3 font-mono text-xs font-semibold text-[#2448CB]">{entry.tool || entry.tool_name || '-'}</td>
                <td class="px-4 py-3 font-mono text-xs text-[#64748B] max-w-md truncate" title={typeof entry.input === 'object' ? JSON.stringify(entry.input) : (entry.input || '')}>
                  {truncate(entry.input, 60)}
                </td>
                <td class="px-4 py-3">
                  <span class="font-semibold text-xs {entry.error || entry.status === 'failed' || entry.outcome === 'error' ? 'text-[#D94B5B]' : 'text-[#219B70]'}">
                    {entry.error ? 'Error' : (entry.status || entry.outcome || 'ok')}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
