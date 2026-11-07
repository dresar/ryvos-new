<script>
  let { visible = $bindable(false), onclose = () => {} } = $props();

  let searchQuery = $state('');
  let inputEl = $state(null);
  let selectedIndex = $state(0);

  const allPages = [
    { route: 'chat', label: 'Chat', desc: 'Send messages to agent' },
    { route: 'dashboard', label: 'Dashboard', desc: 'Overview and metrics' },
    { route: 'sessions', label: 'Sessions', desc: 'Active conversation sessions' },
    { route: 'runs', label: 'Runs', desc: 'Agent run history' },
    { route: 'goals', label: 'Goals', desc: 'Director goal orchestration' },
    { route: 'costs', label: 'Costs', desc: 'Token usage and spending' },
    { route: 'channels', label: 'Channels', desc: 'Channel integrations status' },
    { route: 'integrations', label: 'Integrations', desc: 'OAuth and app connections' },
    { route: 'audit', label: 'Audit Trail', desc: 'Tool execution audit log' },
    { route: 'viking', label: 'Viking Browser', desc: 'Browse Viking context database' },
    { route: 'cron', label: 'Cron Jobs', desc: 'Scheduled recurring tasks' },
    { route: 'config', label: 'Config', desc: 'Edit configuration file' },
    { route: 'settings', label: 'Settings', desc: 'System info and config' },
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
    tabindex="-1"
    class="fixed inset-0 bg-[#0F1729]/40 backdrop-blur-sm z-50 flex items-start justify-center pt-[18vh] p-4"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft-float w-[520px] max-w-full overflow-hidden">
      <div class="flex items-center gap-3 px-4 py-3.5 border-b border-[#DCE4F3]">
        <svg class="w-5 h-5 text-[#778399] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
        <input
          bind:this={inputEl}
          bind:value={searchQuery}
          placeholder="Search pages and features..."
          class="flex-1 bg-transparent text-[#1B2943] text-sm outline-none placeholder:text-[#778399]"
        />
        <kbd class="text-[10px] text-[#64748B] bg-[#EDF1FC] px-2 py-0.5 rounded-md border border-[#DCE4F3] font-mono">ESC</kbd>
      </div>

      <div class="max-h-72 overflow-y-auto p-2 space-y-1">
        {#each filteredPages as page, i}
          <button
            type="button"
            class="w-full flex items-center justify-between px-3.5 py-2.5 rounded-xl text-left transition-colors duration-150 {i === selectedIndex ? 'bg-[#E8EEFF] text-[#2448CB]' : 'text-[#1B2943] hover:bg-[#F4F6FF]'}"
            onclick={() => navigate(page)}
            onmouseenter={() => selectedIndex = i}
          >
            <div>
              <span class="font-semibold text-sm block">{page.label}</span>
              <span class="text-xs text-[#778399]">{page.desc}</span>
            </div>
            {#if i === selectedIndex}
              <svg class="w-4 h-4 text-[#2448CB]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="9 18 15 12 9 6"/>
              </svg>
            {/if}
          </button>
        {/each}
        {#if filteredPages.length === 0}
          <div class="py-6 text-center text-xs text-[#778399]">No pages matching "{searchQuery}"</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
