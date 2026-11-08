<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let runs = $state([]);
  let totalRuns = $state(0);
  let loading = $state(true);
  let error = $state('');
  let note = $state('');
  let currentOffset = $state(0);
  const pageSize = 20;

  async function loadRuns(offset) {
    loading = true;
    error = '';
    currentOffset = offset;
    try {
      const data = await apiFetch('/api/runs?limit=' + pageSize + '&offset=' + offset);
      runs = data.runs || [];
      totalRuns = data.total || 0;
      note = data.note || '';
    } catch (e) {
      error = e.message || 'Failed to fetch runs';
      runs = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => loadRuns(0));

  function formatTime(isoStr) {
    if (!isoStr) return 'N/A';
    try {
      return new Date(isoStr).toLocaleString([], {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return String(isoStr);
    }
  }

  function truncate(s, max) {
    if (!s) return 'None';
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  let totalPages = $derived(Math.ceil(totalRuns / pageSize));
  let currentPage = $derived(Math.floor(currentOffset / pageSize));
</script>

<div class="space-y-4 sm:space-y-6">
  <div>
    <h2 class="text-sm sm:text-base font-bold tracking-tight text-[var(--color-text-main)]">Agent Execution Runs</h2>
    <p class="text-xs text-[var(--color-text-muted)] mt-0.5">Audit log of agent execution cycles, LLM calls, token usage, and costs</p>
  </div>

  {#if loading}
    <div class="neu-card p-6 sm:p-8 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">Loading execution runs...</p>
    </div>
  {:else if error}
    <div class="neu-card p-3 sm:p-5 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-sm sm:text-base font-bold text-[var(--color-danger)] mb-1">Telemetry Unavailable</h3>
      <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">{error}</p>
      <button
        type="button"
        onclick={() => loadRuns(currentOffset)}
        class="btn-soft min-h-[36px] px-3 py-1.5 mt-3 rounded-lg text-xs font-semibold"
      >
        Retry Fetch
      </button>
    </div>
  {:else if runs.length === 0}
    <div class="neu-card p-6 sm:p-8 text-center">
      <p class="text-sm sm:text-base font-semibold text-[var(--color-text-main)] mb-1">No Execution Runs Recorded</p>
      <p class="text-xs text-[var(--color-text-muted)]">{note || 'Runs will appear here once agents receive and process prompts.'}</p>
    </div>
  {:else}
    <div class="neu-card p-0 overflow-hidden">
      <div class="table-scroll overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
            <tr>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Timestamp</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Session</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Model</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Turns</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Tokens</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Cost</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Billing</th>
              <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Status</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[var(--color-border)]">
            {#each runs as run}
              {@const tokens = (run.input_tokens || 0) + (run.output_tokens || 0)}
              {@const cost = '$' + ((run.cost_cents || 0) / 100).toFixed(3)}
              <tr class="hover:bg-[var(--color-bg)] transition-colors">
                <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-muted)] whitespace-nowrap">
                  {formatTime(run.start_time)}
                </td>
                <td class="px-2.5 py-2 font-mono text-xs text-[var(--color-text-main)] whitespace-nowrap" title={run.session_id}>
                  {truncate(run.session_id || '', 16)}
                </td>
                <td class="px-2.5 py-2 font-medium text-xs text-[var(--color-text-main)] whitespace-nowrap">
                  {run.model || 'Default'}
                </td>
                <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-main)]">
                  {run.total_turns || 0}
                </td>
                <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-main)]">
                  {tokens.toLocaleString()}
                </td>
                <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-main)]">
                  {cost}
                </td>
                <td class="px-2.5 py-2 whitespace-nowrap">
                  <span class="inline-flex px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold {run.billing_type === 'subscription' ? 'badge-success' : 'badge-info'}">
                    {run.billing_type || 'API'}
                  </span>
                </td>
                <td class="px-2.5 py-2 whitespace-nowrap">
                  {#if run.status === 'complete' || run.status === 'success'}
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-success">
                      Completed
                    </span>
                  {:else if run.status === 'running'}
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-warning">
                      Running
                    </span>
                  {:else}
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-danger">
                      {run.status || 'Failed'}
                    </span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

    {#if totalPages > 1}
      <div class="flex items-center justify-center gap-1.5 pt-3">
        {#each Array(Math.min(totalPages, 8)) as _, i}
          <button
            type="button"
            onclick={() => loadRuns(i * pageSize)}
            class="min-h-[36px] min-w-[36px] px-2.5 py-1.5 rounded-lg text-xs font-semibold transition-all {i === currentPage ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
          >
            {i + 1}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>