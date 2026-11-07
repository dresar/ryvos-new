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
      const data = await apiFetch('/api/costs?from=' + encodeURIComponent(from) + '&to=' + encodeURIComponent(to) + '&group_by=' + groupBy);
      summary = data.summary || null;
      breakdown = data.breakdown || [];
    } catch (e) {
      error = e.message || 'Failed to fetch cost analytics';
    } finally {
      loading = false;
    }
  }

  onMount(() => loadCosts());

  function capitalize(s) {
    if (!s) return '';
    return s.charAt(0).toUpperCase() + s.slice(1);
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Cost and Token Telemetry</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Realtime aggregation of API spend, tokens, and billing across LLM providers</p>
  </div>

  <div class="neu-card p-4 flex items-center gap-4 flex-wrap">
    <div class="flex items-center gap-2">
      <label for="cost-from-date" class="text-xs font-semibold text-[var(--color-text-muted)]">From</label>
      <input
        id="cost-from-date"
        type="date"
        bind:value={fromDate}
        class="neu-inset min-h-[44px] px-3 py-2 text-xs text-[var(--color-text-main)] rounded-xl"
      />
    </div>

    <div class="flex items-center gap-2">
      <label for="cost-to-date" class="text-xs font-semibold text-[var(--color-text-muted)]">To</label>
      <input
        id="cost-to-date"
        type="date"
        bind:value={toDate}
        class="neu-inset min-h-[44px] px-3 py-2 text-xs text-[var(--color-text-main)] rounded-xl"
      />
    </div>

    <div class="flex items-center gap-2">
      <label for="cost-group-by" class="text-xs font-semibold text-[var(--color-text-muted)]">Group By</label>
      <select
        id="cost-group-by"
        bind:value={groupBy}
        class="neu-inset min-h-[44px] px-3 py-2 text-xs text-[var(--color-text-main)] rounded-xl bg-[var(--color-surface)]"
      >
        <option value="model">Model</option>
        <option value="provider">Provider</option>
        <option value="day">Day</option>
      </select>
    </div>

    <button
      type="button"
      onclick={loadCosts}
      class="btn-primary min-h-[44px] px-5 py-2 text-xs font-semibold rounded-xl ml-auto"
    >
      Update View
    </button>
  </div>

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Computing cost aggregation...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Cost Tracking Offline</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else}
    {#if summary}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          label="Total Cost"
          value={'$' + ((summary.total_cost_cents || 0) / 100).toFixed(2)}
          icon={'<line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/>'}
        />
        <MetricCard
          label="Input Tokens"
          value={(summary.total_input_tokens || 0).toLocaleString()}
          icon={'<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>'}
        />
        <MetricCard
          label="Output Tokens"
          value={(summary.total_output_tokens || 0).toLocaleString()}
          icon={'<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>'}
        />
        <MetricCard
          label="Total Events"
          value={(summary.total_events || 0).toLocaleString()}
          icon={'<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>'}
        />
      </div>
    {/if}

    {#if breakdown.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Usage Recorded</p>
        <p class="text-xs text-[var(--color-text-muted)]">No model calls recorded within the selected date range.</p>
      </div>
    {:else}
      <div class="neu-card p-0 overflow-hidden">
        <div class="table-scroll">
          <table class="w-full text-left text-sm">
            <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
              <tr>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                  {capitalize(groupBy)}
                </th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                  Cost (USD)
                </th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                  Input Tokens
                </th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">
                  Output Tokens
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
              {#each breakdown as row}
                <tr class="hover:bg-[var(--color-bg)] transition-colors">
                  <td class="px-4 py-3 font-semibold text-xs text-[var(--color-text-main)] whitespace-nowrap">
                    {row.key}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs tabular-nums text-[var(--color-text-main)]">
                    ${((row.cost_cents || 0) / 100).toFixed(3)}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs tabular-nums text-[var(--color-text-muted)]">
                    {(row.input_tokens || 0).toLocaleString()}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs tabular-nums text-[var(--color-text-muted)]">
                    {(row.output_tokens || 0).toLocaleString()}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  {/if}
</div>