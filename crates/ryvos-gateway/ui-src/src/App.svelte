<script>
  import { onMount, onDestroy } from 'svelte';
  import { getApiKey, setApiKey } from './lib/api.js';
  import { connect, disconnect } from './lib/ws.js';
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

  let unsubAuth = () => {};

  function parseHash() {
    const hash = window.location.hash || '#/chat';
    const parts = hash.replace('#/', '').split('/');
    currentRoute = parts[0] || 'chat';
    routeParam = decodeURIComponent(parts.slice(1).join('/') || '');
  }

  function handleLogin(e) {
    const key = e.key;
    setApiKey(key);
    authenticated.set(true);
    connect(key);
    parseHash();
  }

  function handleKeydown(e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
  }

  onMount(() => {
    unsubAuth = authenticated.subscribe(v => {
      isAuthenticated = v;
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
    window.removeEventListener('hashchange', parseHash);
    window.removeEventListener('keydown', handleKeydown);
    disconnect();
  });
</script>

{#if !isAuthenticated}
  <LoginOverlay onlogin={handleLogin} />
{:else}
  <div class="flex h-screen bg-[#EDF1FC] overflow-hidden">
    <Sidebar {currentRoute} bind:open={sidebarOpen} />

    <div class="flex-1 flex flex-col min-w-0 h-screen overflow-hidden">
      <header class="md:hidden flex items-center justify-between px-4 py-3 bg-white/90 backdrop-blur-md border-b border-[#DCE4F3] z-30">
        <button
          type="button"
          aria-label="Open sidebar menu"
          class="btn-surface p-2 rounded-xl flex items-center justify-center text-[#1B2943]"
          onclick={() => sidebarOpen = true}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>
          </svg>
        </button>

        <span class="font-heading font-bold text-base text-[#1B2943] capitalize">{currentRoute}</span>

        <button
          type="button"
          aria-label="Search"
          class="btn-surface p-2 rounded-xl flex items-center justify-center text-[#64748B]"
          onclick={() => showCommandPalette = true}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
        </button>
      </header>

      <main class="flex-1 overflow-y-auto p-4 md:p-8 pb-24 md:pb-8">
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
      </main>
    </div>

    <MobileBottomNav {currentRoute} onToggleMenu={() => sidebarOpen = !sidebarOpen} />
  </div>

  <CommandPalette bind:visible={showCommandPalette} onclose={() => showCommandPalette = false} />
{/if}
