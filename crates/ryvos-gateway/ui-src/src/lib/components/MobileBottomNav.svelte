<script>
  let { currentRoute = 'chat', onToggleMenu = () => {} } = $props();

  const primaryTabs = [
    { route: 'chat', label: 'Chat', icon: 'message' },
    { route: 'dashboard', label: 'Overview', icon: 'grid' },
    { route: 'sessions', label: 'Sessions', icon: 'layers' },
    { route: 'runs', label: 'Runs', icon: 'activity' },
  ];

  function isActive(route) {
    return currentRoute === route;
  }
</script>

<nav class="fixed bottom-0 left-0 right-0 z-40 md:hidden bg-white/95 backdrop-blur-md border-t border-[#DCE4F3] shadow-soft-float pb-[env(safe-area-inset-bottom)]">
  <div class="flex items-center justify-around px-2 py-1.5 h-14">
    {#each primaryTabs as tab}
      <a
        href="#/{tab.route}"
        class="flex flex-col items-center justify-center flex-1 py-1 px-1 rounded-xl transition-all duration-150 {isActive(tab.route) ? 'text-[#2448CB] font-semibold bg-[#E8EEFF]/80' : 'text-[#64748B] hover:text-[#1B2943]'}"
      >
        {#if tab.icon === 'message'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
          </svg>
        {:else if tab.icon === 'grid'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
          </svg>
        {:else if tab.icon === 'layers'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>
          </svg>
        {:else if tab.icon === 'activity'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
          </svg>
        {/if}
        <span class="text-[10px] tracking-tight mt-0.5">{tab.label}</span>
      </a>
    {/each}
    <button
      type="button"
      onclick={onToggleMenu}
      aria-label="Toggle navigation drawer"
      class="flex flex-col items-center justify-center flex-1 py-1 px-1 rounded-xl text-[#64748B] hover:text-[#1B2943] transition-all duration-150"
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="4" y1="6" x2="20" y2="6"/><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="18" x2="20" y2="18"/>
      </svg>
      <span class="text-[10px] tracking-tight mt-0.5">More</span>
    </button>
  </div>
</nav>
