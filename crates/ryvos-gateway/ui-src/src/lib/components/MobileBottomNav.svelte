<script>
  let { currentRoute = 'chat' } = $props();

  const primaryTabs = [
    { route: 'chat', label: 'Chat', icon: 'message' },
    { route: 'dashboard', label: 'Dashboard', icon: 'grid' },
    { route: 'runs', label: 'Runs', icon: 'activity' },
    { route: 'goals', label: 'Goals', icon: 'target' },
    { route: 'settings', label: 'Settings', icon: 'settings' },
  ];

  function isActive(route) {
    return currentRoute === route;
  }
</script>

<nav
  aria-label="Mobile navigation"
  class="fixed bottom-0 left-0 right-0 z-40 md:hidden bg-[var(--color-surface)]/95 backdrop-blur-md border-t border-[var(--color-border)] pb-[env(safe-area-inset-bottom)] shadow-lg"
>
  <div class="flex items-center justify-around px-2 h-12 sm:h-14">
    {#each primaryTabs as tab}
      <a
        href="#/{tab.route}"
        aria-current={isActive(tab.route) ? 'page' : undefined}
        class="flex flex-col items-center justify-center flex-1 min-h-[40px] py-1 px-1 rounded-xl transition-all duration-150 {isActive(tab.route) ? 'text-[var(--color-primary)] font-bold bg-[var(--color-primary-soft)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text)]'}"
      >
        {#if tab.icon === 'message'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
          </svg>
        {:else if tab.icon === 'grid'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
          </svg>
        {:else if tab.icon === 'activity'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
          </svg>
        {:else if tab.icon === 'target'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>
          </svg>
        {:else if tab.icon === 'settings'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
          </svg>
        {/if}
        <span class="text-[10px] font-medium tracking-tight mt-0.5">{tab.label}</span>
      </a>
    {/each}
  </div>
</nav>
