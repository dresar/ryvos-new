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
      const data = await apiFetch(`/api/runs?limit=${pageSize}&offset=${offset}`);
      runs = data.runs || [];
      totalRuns = data.total || 0;
      note = data.note || '';
    } catch (e) {
      error = e.message;
      runs = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => loadRuns(0));

  function formatTime(isoStr) {
    if (!isoStr) return '-';
    return new Date(isoStr).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }

  function truncate(s, max) {
    if (!s) return '';
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  let totalPages = $derived(Math.ceil(totalRuns / pageSize));
  let currentPage = $derived(Math.floor(currentOffset / pageSize));
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Run History</h2>
    <p class="text-[#778399] text-sm mt-1">Each run is one agent execution cycle: a complete request, response, and tool execution loop.</p>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading runs...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Cost tracking not configured</p>
    </div>
  {:else if runs.length === 0}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">{note || 'No runs recorded yet'}</p>
    </div>
  {:else}
    <div class="card-soft overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
              {#each ['Time', 'Session', 'Model', 'Turns', 'Tokens', 'Cost', 'Type', 'Status'] as col}
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                  {col}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody class="divide-y divide-[#DCE4F3]">
            {#each runs as run}
              {@const tokens = (run.input_tokens || 0) + (run.output_tokens || 0)}
              {@const cost = '$' + ((run.cost_cents || 0) / 100).toFixed(3)}
              <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                <td class="px-4 py-3 font-mono text-xs text-[#778399] tabular-nums">{formatTime(run.start_time)}</td>
                <td class="px-4 py-3 font-mono text-xs text-[#778399]">{truncate(run.session_id || '', 14)}</td>
                <td class="px-4 py-3 font-medium text-[#1B2943]">{run.model || '-'}</td>
                <td class="px-4 py-3 text-[#1B2943] tabular-nums">{run.total_turns || 0}</td>
                <td class="px-4 py-3 text-[#1B2943] tabular-nums">{tokens.toLocaleString()}</td>
                <td class="px-4 py-3 font-mono text-[#1B2943] tabular-nums">{cost}</td>
                <td class="px-4 py-3">
                  <span class="inline-flex px-2.5 py-0.5 text-xs font-semibold rounded-full
                    {run.billing_type === 'subscription' ? 'bg-[#219B70]/10 text-[#219B70]' : 'bg-[#2448CB]/10 text-[#2448CB]'}">
                    {run.billing_type || 'api'}
                  </span>
                </td>
                <td class="px-4 py-3">
                  <span class="font-semibold text-xs
                    {run.status === 'complete' ? 'text-[#219B70]' :
                     run.status === 'running' ? 'text-[#C58A2A]' :
                     run.status === 'error' ? 'text-[#D94B5B]' : 'text-[#778399]'}">
                    {run.status || '-'}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

    {#if totalPages > 1}
      <div class="flex justify-center gap-1.5 mt-5">
        {#each Array(Math.min(totalPages, 10)) as _, i}
          <button
            type="button"
            onclick={() => loadRuns(i * pageSize)}
            class="px-3 py-1.5 text-xs font-semibold rounded-xl transition-all duration-150
              {i === currentPage
                ? 'bg-[#2448CB] text-white shadow-soft-sm'
                : 'btn-surface text-[#778399] hover:text-[#1B2943]'}"
          >
            {i + 1}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
