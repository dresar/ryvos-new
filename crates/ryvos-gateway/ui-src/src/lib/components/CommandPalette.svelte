<script>
  let { visible = $bindable(false), onclose = () => {} } = $props();

  let searchQuery = $state('');
  let inputEl = $state(null);
  let selectedIndex = $state(0);

  const allPages = [
    { route: 'chat', label: 'Chat', desc: 'Interact directly with autonomous agent' },
    { route: 'dashboard', label: 'Dashboard', desc: 'Telemetry overview and system status' },
    { route: 'sessions', label: 'Sessions', desc: 'Active conversational contexts and history' },
    { route: 'runs', label: 'Runs', desc: 'Detailed log of previous execution runs' },
    { route: 'goals', label: 'Goals', desc: 'Autonomous director goal execution and judge telemetry' },
    { route: 'costs', label: 'Costs', desc: 'Token usage metrics and budget tracking' },
    { route: 'channels', label: 'Channels', desc: 'Telegram, Discord, and Slack adapters' },
    { route: 'integrations', label: 'Tools', desc: 'MCP servers and external service connections' },
    { route: 'audit', label: 'Audit Trail', desc: 'Security log, safety lessons, decisions, failures' },
    { route: 'viking', label: 'Viking Context', desc: 'Hierarchical L0, L1, L2 memory viewer and search' },
    { route: 'cron', label: 'Cron Jobs', desc: 'Scheduled background triggers and heartbeat health' },
    { route: 'config', label: 'Config Editor', desc: 'Live configuration file manager' },
    { route: 'settings', label: 'Settings', desc: 'Model provider switching and system preferences' },
  ];

  let filteredPages = $derived(
    searchQuery.trim()
      ? allPages.filter(p =>
          p.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.desc.toLowerCase().includes(searchQuery.toLowerCase()))
      : allPages
  );

  $effect(() => {
    if (selectedIndex >= filteredPages.length) {
      selectedIndex = Math.max(0, filteredPages.length - 1);
    }
  });

  $effect(() => {
    if (visible && inputEl) {
      setTimeout(() => inputEl?.focus(), 50);
    }
  });

  function handleKeydown(e) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredPages.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter' && filteredPages.length > 0) {
      navigate(filteredPages[selectedIndex]);
    }
  }

  function navigate(page) {
    window.location.hash = '#/' + page.route;
    close();
  }

  function close() {
    searchQuery = '';
    selectedIndex = 0;
    visible = false;
    onclose();
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) close();
  }
</script>

{#if visible}
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Command search dialog"
    tabindex="-1"
    class="fixed inset-0 bg-[#0F1729]/50 backdrop-blur-sm z-50 flex items-start justify-center pt-[15vh] p-4"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="neu-focal bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl w-[560px] max-w-full overflow-hidden shadow-2xl">
      <div class="flex items-center gap-3 px-4 py-3.5 border-b border-[var(--color-border)] bg-[var(--color-surface-raised)]">
        <svg class="w-5 h-5 text-[var(--color-primary)] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <label for="command-palette-search" class="sr-only">Search routes</label>
        <input
          id="command-palette-search"
          bind:this={inputEl}
          bind:value={searchQuery}
          placeholder="Search control plane views and tools..."
          class="flex-1 bg-transparent text-[var(--color-text)] text-sm outline-none placeholder:text-[var(--color-text-secondary)] font-medium"
        />
        <kbd class="text-[10px] text-[var(--color-text-secondary)] bg-[var(--color-surface-inset)] px-2 py-0.5 rounded-md border border-[var(--color-border)] font-mono">
          ESC
        </kbd>
      </div>

      <div class="max-h-80 overflow-y-auto p-2 space-y-1">
        {#if filteredPages.length === 0}
          <div class="p-6 text-center text-sm text-[var(--color-text-secondary)]">
            No matching routes found
          </div>
        {:else}
          {#each filteredPages as page, i}
            <button
              type="button"
              onclick={() => navigate(page)}
              class="w-full text-left px-3.5 py-2.5 rounded-xl text-sm flex items-center justify-between transition-colors min-h-[44px]
                {i === selectedIndex
                  ? 'bg-[var(--color-primary-soft)] text-[var(--color-primary)] font-semibold'
                  : 'text-[var(--color-text)] hover:bg-[var(--color-surface-raised)]'}"
            >
              <div>
                <span class="font-bold">{page.label}</span>
                <p class="text-xs text-[var(--color-text-secondary)] font-normal">{page.desc}</p>
              </div>
              <span class="text-xs font-mono text-[var(--color-text-secondary)] opacity-60">#{page.route}</span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
