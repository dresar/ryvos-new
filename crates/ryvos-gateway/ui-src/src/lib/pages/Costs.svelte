<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';
  import MetricCard from '../components/MetricCard.svelte';

  let summary = $state(null);
  let breakdown = $state([]);
  let loading = $state(true);
  let error = $state('');

  let now = new Date();
  let thirtyDaysAgo = new Date(now.getTime() - 30 * 86400000);
  let fromDate = $state(thirtyDaysAgo.toISOString().split('T')[0]);
  let toDate = $state(now.toISOString().split('T')[0]);
  let groupBy = $state('model');

  async function loadCosts() {
    loading = true;
    error = '';
    try {
      const from = fromDate + 'T00:00:00Z';
      const to = toDate + 'T23:59:59Z';
      const data = await apiFetch(`/api/costs?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}&group_by=${groupBy}`);
      summary = data.summary || null;
      breakdown = data.breakdown || [];
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  onMount(() => loadCosts());

  function capitalize(s) {
    return s.charAt(0).toUpperCase() + s.slice(1);
  }
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Cost Analysis</h2>
    <p class="text-[#778399] text-sm mt-1">Token consumption and cost aggregation across providers</p>
  </div>

  <div class="card-soft p-4 mb-6 flex items-center gap-4 flex-wrap">
    <label class="flex items-center gap-2 text-xs text-[#64748B] font-medium">
      From
      <input
        type="date"
        bind:value={fromDate}
        class="input-soft px-3 py-1.5 text-xs text-[#1B2943]"
      />
    </label>
    <label class="flex items-center gap-2 text-xs text-[#64748B] font-medium">
      To
      <input
        type="date"
        bind:value={toDate}
        class="input-soft px-3 py-1.5 text-xs text-[#1B2943]"
      />
    </label>
    <label class="flex items-center gap-2 text-xs text-[#64748B] font-medium">
      Group by
      <select
        bind:value={groupBy}
        class="input-soft px-3 py-1.5 text-xs text-[#1B2943]"
      >
        <option value="model">Model</option>
        <option value="provider">Provider</option>
        <option value="day">Day</option>
      </select>
    </label>
    <button
      type="button"
      onclick={loadCosts}
      class="btn-primary px-4 py-1.5 text-xs font-semibold ml-auto"
    >
      Refresh
    </button>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading cost data...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Cost tracking not configured</p>
    </div>
  {:else}
    {#if summary}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <MetricCard label="Total Cost" value={'$' + ((summary.total_cost_cents || 0) / 100).toFixed(2)} icon={'<line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/>'} />
        <MetricCard label="Input Tokens" value={(summary.total_input_tokens || 0).toLocaleString()} icon={'<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>'} />
        <MetricCard label="Output Tokens" value={(summary.total_output_tokens || 0).toLocaleString()} icon={'<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>'} />
        <MetricCard label="Events" value={(summary.total_events || 0).toLocaleString()} icon={'<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>'} />
      </div>
    {/if}

    {#if breakdown.length === 0}
      <div class="card-soft p-8 text-center">
        <p class="text-[#778399] text-sm">No cost data recorded for this period</p>
      </div>
    {:else}
      <div class="card-soft overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                  {capitalize(groupBy)}
                </th>
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Cost</th>
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Input Tokens</th>
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">Output Tokens</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[#DCE4F3]">
              {#each breakdown as row}
                <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                  <td class="px-4 py-3 font-semibold text-[#1B2943]">{row.key}</td>
                  <td class="px-4 py-3 font-mono text-[#1B2943] tabular-nums">${((row.cost_cents || 0) / 100).toFixed(3)}</td>
                  <td class="px-4 py-3 text-[#778399] tabular-nums">{(row.input_tokens || 0).toLocaleString()}</td>
                  <td class="px-4 py-3 text-[#778399] tabular-nums">{(row.output_tokens || 0).toLocaleString()}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  {/if}
</div>
