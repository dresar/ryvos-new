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
      error = e.message || 'Failed to fetch communication channels';
    } finally {
      loading = false;
    }
  });
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Communication Channels</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Status of messaging adapters, gateways, and external integrations</p>
  </div>

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Checking channel statuses...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Channel Query Error</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else if channels.length === 0}
    <div class="neu-card p-12 text-center">
      <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Channels Configured</p>
      <p class="text-xs text-[var(--color-text-muted)] max-w-sm mx-auto">Telegram, Discord, and Slack adapters configured in config.toml will appear here.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each channels as channel}
        {@const statusLower = (channel.status || '').toLowerCase()}
        {@const isOnline = statusLower === 'connected' || statusLower === 'active' || statusLower === 'ok'}
        {@const isErr = statusLower === 'error' || statusLower === 'failed'}
        <div class="neu-card p-5 space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-bold text-[var(--color-text-main)]">{channel.name || 'Unknown Channel'}</h3>
            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-full {isOnline ? 'badge-success' : isErr ? 'badge-danger' : 'badge-neutral'}">
              <span class="w-1.5 h-1.5 rounded-full {isOnline ? 'bg-[var(--color-success)]' : isErr ? 'bg-[var(--color-danger)]' : 'bg-[var(--color-text-muted)]'}"></span>
              {channel.status || 'Offline'}
            </span>
          </div>
          {#if channel.type}
            <div class="text-xs text-[var(--color-text-muted)]">
              Protocol: <span class="font-mono text-[var(--color-text-main)]">{channel.type}</span>
            </div>
          {/if}
          {#if channel.description}
            <p class="text-xs text-[var(--color-text-muted)] leading-relaxed">{channel.description}</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>