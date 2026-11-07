<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let channels = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      const data = await apiFetch('/api/channels');
      let raw = data.channels || data || [];
      if (!Array.isArray(raw)) {
        channels = Object.entries(raw).map(([name, info]) => ({
          name,
          ...(typeof info === 'object' ? info : { status: info }),
        }));
      } else {
        channels = raw;
      }
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  });
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Channels</h2>
    <p class="text-[#778399] text-sm mt-1">Integration messaging adapters and communication channels</p>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading channels...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Channels endpoint not available: {error}</p>
    </div>
  {:else if channels.length === 0}
    <div class="card-soft p-12 text-center">
      <div class="w-12 h-12 rounded-2xl bg-[#E8EEFF] text-[#2448CB] flex items-center justify-center mx-auto mb-3">
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 010 8.49m-8.48-.01a6 6 0 010-8.49m11.31-2.82a10 10 0 010 14.14m-14.14 0a10 10 0 010-14.14"/></svg>
      </div>
      <h3 class="text-base font-semibold text-[#1B2943] mb-1">No channels configured</h3>
      <p class="text-xs text-[#778399] max-w-sm mx-auto">Telegram, Discord, and WhatsApp adapters configured in config.toml will appear here.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each channels as channel}
        <div class="card-soft p-5 transition-all duration-200 hover:shadow-soft-float">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-bold text-[#1B2943]">{channel.name || 'Unknown'}</h3>
            <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-semibold rounded-full
              {channel.status === 'connected' || channel.status === 'active' || channel.status === 'ok'
                ? 'bg-[#219B70]/10 text-[#219B70]'
                : channel.status === 'error'
                  ? 'bg-[#D94B5B]/10 text-[#D94B5B]'
                  : 'bg-[#F4F6FF] text-[#778399]'}">
              <span class="w-2 h-2 rounded-full
                {channel.status === 'connected' || channel.status === 'active' || channel.status === 'ok'
                  ? 'bg-[#219B70]'
                  : channel.status === 'error'
                    ? 'bg-[#D94B5B]'
                    : 'bg-[#778399]'}"></span>
              {channel.status || 'unknown'}
            </span>
          </div>
          {#if channel.type}
            <p class="text-xs text-[#64748B]">Type: <span class="font-mono text-[#1B2943]">{channel.type}</span></p>
          {/if}
          {#if channel.description}
            <p class="text-xs text-[#778399] mt-1">{channel.description}</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
