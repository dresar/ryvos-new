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

<div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-5">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-sm font-bold text-[#1B2943]">Activity Feed</h3>
    <span class="text-xs px-2.5 py-0.5 bg-[#E8EEFF] text-[#2448CB] font-semibold rounded-full shadow-sm">Live</span>
  </div>

  <div class="max-h-[480px] overflow-y-auto space-y-1">
    {#if feed.length === 0}
      <p class="text-[#778399] text-sm text-center py-8">Events from heartbeats, agent runs, and cron jobs will appear here.</p>
    {:else}
      {#each feed as item}
        <div class="flex items-start gap-2.5 py-2 px-2.5 rounded-xl hover:bg-[#F4F6FF] transition-colors duration-150 text-xs">
          <span class="w-2 h-2 rounded-full mt-1 shrink-0
            {item.kind === 'run_error' || item.kind === 'budget_exceeded' ? 'bg-[#D94B5B]' :
             item.kind === 'budget_warning' ? 'bg-[#C58A2A]' : 'bg-[#2448CB]'}"></span>
          <span class="text-[#778399] min-w-[64px] font-mono tabular-nums">{item.time}</span>
          <span class="flex-1
            {item.kind === 'run_error' || item.kind === 'budget_exceeded' ? 'text-[#D94B5B] font-medium' :
             item.kind === 'budget_warning' ? 'text-[#C58A2A] font-medium' : 'text-[#1B2943]'}">
            {item.detail}
          </span>
          {#if item.session}
            <span class="text-[#2448CB]/80 font-mono text-[11px] bg-[#E8EEFF] px-1.5 py-0.5 rounded-md">{item.session}</span>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
