<script>
  import { onMount, onDestroy } from 'svelte';
  import { activityFeed } from '../ws.js';

  let feed = $state([]);
  let unsub = () => {};

  onMount(() => {
    unsub = activityFeed.subscribe(v => {
      feed = v;
    });
  });

  onDestroy(() => {
    unsub();
  });
</script>

<div class="neu-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-5 shadow-sm">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-sm font-bold text-[var(--color-text)] m-0">Live Activity Stream</h3>
    <span class="text-xs px-2.5 py-0.5 bg-[var(--color-primary-soft)] text-[var(--color-primary)] font-bold rounded-full">
      Telemetry
    </span>
  </div>

  <div class="max-h-[440px] overflow-y-auto space-y-1.5 pr-1">
    {#if feed.length === 0}
      <div class="text-[var(--color-text-secondary)] text-sm text-center py-12">
        Events from heartbeat checks, tool calls, and cron schedules will appear here in realtime.
      </div>
    {:else}
      {#each feed as item}
        <div class="flex items-start gap-2.5 py-2 px-3 rounded-xl hover:bg-[var(--color-surface-raised)] transition-colors duration-150 text-xs border border-transparent hover:border-[var(--color-border)]">
          <span class="w-2 h-2 rounded-full mt-1.5 shrink-0
            {item.kind === 'run_error' || item.kind === 'budget_exceeded' ? 'bg-[var(--color-danger)]' :
             item.kind === 'budget_warning' || item.kind === 'guardian_stall' ? 'bg-[var(--color-warning)]' :
             'bg-[var(--color-primary)]'}"></span>
          <span class="text-[var(--color-text-secondary)] min-w-[64px] font-mono tabular-nums">{item.time}</span>
          <span class="flex-1
            {item.kind === 'run_error' || item.kind === 'budget_exceeded' ? 'text-[var(--color-danger)] font-medium' :
             item.kind === 'budget_warning' ? 'text-[var(--color-warning)] font-medium' :
             'text-[var(--color-text)] font-medium'}">
            {item.detail}
          </span>
          {#if item.session}
            <span class="text-[var(--color-primary)] font-mono text-[11px] bg-[var(--color-primary-soft)] px-1.5 py-0.5 rounded-md">
              {item.session}
            </span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
