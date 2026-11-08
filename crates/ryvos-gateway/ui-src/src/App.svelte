<script>
  import { onMount, onDestroy } from 'svelte';
  import { getApiKey, setApiKey } from './lib/api.js';
  import { connect, disconnect, connectionStatus } from './lib/ws.js';
  import { authenticated } from './lib/stores.js';

  import Sidebar from './lib/components/Sidebar.svelte';
  import MobileBottomNav from './lib/components/MobileBottomNav.svelte';
  import LoginOverlay from './lib/components/LoginOverlay.svelte';
  import CommandPalette from './lib/components/CommandPalette.svelte';

  import Dashboard from './lib/pages/Dashboard.svelte';
  import Chat from './lib/pages/Chat.svelte';
  import Sessions from './lib/pages/Sessions.svelte';
  import Runs from './lib/pages/Runs.svelte';
  import Costs from './lib/pages/Costs.svelte';
  import Settings from './lib/pages/Settings.svelte';
  import AuditTrail from './lib/pages/AuditTrail.svelte';
  import VikingBrowser from './lib/pages/VikingBrowser.svelte';
  import ConfigEditor from './lib/pages/ConfigEditor.svelte';
  import Channels from './lib/pages/Channels.svelte';
  import Integrations from './lib/pages/Integrations.svelte';
  import CronManager from './lib/pages/Graph.svelte';
  import Goals from './lib/pages/Goals.svelte';

  let isAuthenticated = $state(false);
  let currentRoute = $state('chat');
  let routeParam = $state('');
  let showCommandPalette = $state(false);
  let sidebarOpen = $state(false);
  let theme = $state('light');
  let connState = $state('disconnected');

  let unsubAuth = () => {};
  let unsubConn = () => {};

  function parseHash() {
    const hash = window.location.hash || '#/chat';
    const parts = hash.replace('#/', '').split('/');
    let primary = parts[0] || 'chat';
    if (primary === 'memory') {
      primary = 'viking';
    }
    currentRoute = primary;
    routeParam = decodeURIComponent(parts.slice(1).join('/') || '');
  }

  function handleLogin(e) {
    const key = e.key;
    setApiKey(key);
    authenticated.set(true);
    connect(key);
    parseHash();
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    localStorage.setItem('ryvos_theme', theme);
    document.documentElement.setAttribute('data-theme', theme);
  }

  function handleKeydown(e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
  }

  const pageTitles = {
    chat: 'Agent Chat',
    dashboard: 'Control Plane Dashboard',
    sessions: 'Conversation Sessions',
    runs: 'Execution Runs',
    goals: 'Autonomous Goals',
    costs: 'Resource Costs',
    channels: 'Channel Integrations',
    integrations: 'External Tools',
    audit: 'Audit and Safety Trail',
    viking: 'Viking Memory Browser',
    cron: 'Scheduled Cron Jobs',
    config: 'Runtime Configuration',
    settings: 'Cluster Settings',
  };

  let pageTitle = $derived(pageTitles[currentRoute] || 'Agent Chat');

  onMount(() => {
    const savedTheme = localStorage.getItem('ryvos_theme') || 'light';
    theme = savedTheme;
    document.documentElement.setAttribute('data-theme', theme);

    unsubAuth = authenticated.subscribe(v => {
      isAuthenticated = v;
    });

    unsubConn = connectionStatus.subscribe(v => {
      connState = v;
    });

    const key = getApiKey();
    if (key || key === '') {
      authenticated.set(true);
      connect(key);
    }
    parseHash();
    window.addEventListener('hashchange', parseHash);
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    unsubAuth();
    unsubConn();
    window.removeEventListener('hashchange', parseHash);
    window.removeEventListener('keydown', handleKeydown);
    disconnect();
  });
</script>

<a href="#main-content" class="skip-link">Skip to main content</a>

{#if !isAuthenticated}
  <LoginOverlay onlogin={handleLogin} />
{:else}
  <div class="flex h-screen overflow-hidden neu-shell">
    <Sidebar {currentRoute} bind:open={sidebarOpen} />

    <div class="flex-1 flex flex-col min-w-0 h-screen overflow-hidden">
      <header
        aria-label="Application header"
        class="h-12 sm:h-14 shrink-0 sticky top-0 z-30 px-2.5 sm:px-6 flex items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-surface)]/85 backdrop-blur-md"
      >
        <div class="flex items-center gap-3 min-w-0">
          <button
            type="button"
            aria-label="Open navigation menu"
            aria-expanded={sidebarOpen}
            class="lg:hidden min-h-[36px] min-w-[36px] btn-soft flex items-center justify-center p-1.5 rounded-lg text-[var(--color-text)]"
            onclick={() => sidebarOpen = true}
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          </button>

          <h1 class="text-sm sm:text-base font-bold tracking-tight text-[var(--color-text)] truncate m-0">
            {pageTitle}
          </h1>
        </div>

        <div class="flex items-center gap-2 sm:gap-3">
          <div
            class="hidden sm:inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold neu-inset"
            title="WebSocket connection status"
          >
            <span
              class="w-2 h-2 rounded-full {connState === 'connected' ? 'bg-[var(--color-success)]' : connState === 'error' ? 'bg-[var(--color-danger)]' : 'bg-[var(--color-text-secondary)]'}"
            ></span>
            <span class="text-[var(--color-text)] capitalize">{connState}</span>
          </div>

          <button
            type="button"
            aria-label="Search and quick command palette"
            class="min-h-[36px] min-w-[36px] btn-soft flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-xs text-[var(--color-text-secondary)]"
            onclick={() => showCommandPalette = true}
          >
            <svg class="w-4 h-4 text-[var(--color-primary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
            </svg>
            <span class="hidden md:inline font-mono text-[11px]">Ctrl+K</span>
          </button>

          <button
            type="button"
            aria-label="Toggle visual theme"
            class="min-h-[36px] min-w-[36px] btn-soft flex items-center justify-center p-1.5 rounded-lg text-[var(--color-text)]"
            onclick={toggleTheme}
          >
            {#if theme === 'light'}
              <svg class="w-4 h-4 text-[var(--color-primary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
              </svg>
            {:else}
              <svg class="w-4 h-4 text-[var(--color-warning)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
              </svg>
            {/if}
          </button>
        </div>
      </header>

      <main
        id="main-content"
        tabindex="-1"
        class="flex-1 overflow-y-auto p-2 sm:p-4 lg:p-6 pb-[calc(4.5rem+env(safe-area-inset-bottom,0px))] md:pb-6 outline-none"
      >
        {#if currentRoute === 'chat'}
          <Chat sessionId={routeParam} />
        {:else if currentRoute === 'dashboard'}
          <Dashboard />
        {:else if currentRoute === 'sessions'}
          <Sessions />
        {:else if currentRoute === 'runs'}
          <Runs />
        {:else if currentRoute === 'costs'}
          <Costs />
        {:else if currentRoute === 'settings'}
          <Settings />
        {:else if currentRoute === 'audit'}
          <AuditTrail />
        {:else if currentRoute === 'viking'}
          <VikingBrowser />
        {:else if currentRoute === 'config'}
          <ConfigEditor />
        {:else if currentRoute === 'channels'}
          <Channels />
        {:else if currentRoute === 'integrations'}
          <Integrations />
        {:else if currentRoute === 'cron'}
          <CronManager />
        {:else if currentRoute === 'goals'}
          <Goals />
        {:else}
          <Chat sessionId={routeParam} />
        {/if}

        <footer aria-label="Application footer" class="mt-12 pt-6 border-t border-[var(--color-border)] flex flex-col sm:flex-row items-center justify-between gap-3 text-xs text-[var(--color-text-secondary)]">
          <div class="flex items-center gap-2">
            <span class="font-bold text-[var(--color-text)]">Ryvos Control Plane</span>
            <span>v0.9.0</span>
            <span>&bull;</span>
            <span class="capitalize">{connState}</span>
          </div>
          <div class="flex items-center gap-4">
            <a href="#/audit" class="hover:text-[var(--color-primary)] transition-colors">Audit Log</a>
            <a href="#/settings" class="hover:text-[var(--color-primary)] transition-colors">Settings</a>
            <a href="#/viking" class="hover:text-[var(--color-primary)] transition-colors">Viking Context</a>
          </div>
        </footer>
      </main>
    </div>

    <MobileBottomNav {currentRoute} />
  </div>

  <CommandPalette bind:visible={showCommandPalette} onclose={() => showCommandPalette = false} />
{/if}
