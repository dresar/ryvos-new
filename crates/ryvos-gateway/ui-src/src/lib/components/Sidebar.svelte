<script>
  import { onMount, onDestroy } from 'svelte';
  import { connectionStatus, disconnect } from '../ws.js';
  import { clearApiKey, apiFetch } from '../api.js';
  import { authenticated } from '../stores.js';
  import logoUrl from '../../assets/logo.png';

  let { currentRoute = 'chat', open = $bindable(true) } = $props();

  let status = $state('disconnected');
  let version = $state('');
  let unsub = () => {};

  onMount(async () => {
    unsub = connectionStatus.subscribe(v => {
      status = v;
    });
    try {
      const data = await apiFetch('/api/health');
      version = data.version || '';
    } catch {}
  });

  onDestroy(() => {
    unsub();
  });

  const navItems = [
    { route: 'chat',       label: 'Chat',            icon: 'message' },
    { route: 'dashboard',  label: 'Dashboard',       icon: 'grid' },
    { route: 'sessions',   label: 'Sessions',        icon: 'layers' },
    { route: 'runs',       label: 'Runs',            icon: 'activity' },
    { route: 'goals',      label: 'Goals',           icon: 'target' },
    { route: 'costs',      label: 'Costs',           icon: 'dollar' },
    { route: 'channels',   label: 'Channels',        icon: 'radio' },
    { route: 'integrations', label: 'Integrations',  icon: 'plug' },
    { route: 'audit',      label: 'Audit Trail',     icon: 'shield' },
    { route: 'viking',     label: 'Viking Browser',  icon: 'database' },
    { route: 'cron',       label: 'Cron Jobs',       icon: 'clock' },
    { route: 'config',     label: 'Config',          icon: 'file' },
    { route: 'settings',   label: 'Settings',        icon: 'settings' },
  ];

  function handleLogout() {
    disconnect();
    clearApiKey();
    authenticated.set(false);
  }

  function isActive(route) {
    return route === currentRoute;
  }

  function handleNavClick() {
    if (window.innerWidth < 768) {
      open = false;
    }
  }
</script>

{#if open}
  <div
    role="button"
    tabindex="0"
    aria-label="Close navigation drawer"
    class="fixed inset-0 bg-[#0F1729]/40 backdrop-blur-sm z-40 md:hidden"
    onclick={() => open = false}
    onkeydown={(e) => { if (e.key === 'Escape' || e.key === 'Enter') open = false; }}
  ></div>
{/if}

<aside
  class="w-64 bg-white border-r border-[#DCE4F3] flex flex-col h-screen shrink-0
    fixed md:relative z-50 transition-transform duration-200 ease-out shadow-soft md:shadow-none
    {open ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}"
>
  <div class="flex items-center gap-3 px-5 py-4 border-b border-[#DCE4F3]">
    <div class="w-8 h-8 rounded-xl bg-[#E8EEFF] flex items-center justify-center shadow-soft-sm">
      <img src={logoUrl} alt="Ryvos" class="w-5 h-5 object-contain" />
    </div>
    <span class="text-lg font-heading font-bold tracking-tight text-[#1B2943]">Ryvos</span>
    {#if version}
      <span class="text-[10px] font-bold uppercase tracking-wider text-[#2448CB] bg-[#E8EEFF] px-2 py-0.5 rounded-full">v{version}</span>
    {/if}
    <button
      type="button"
      aria-label="Close menu"
      class="ml-auto md:hidden w-8 h-8 rounded-lg flex items-center justify-center text-[#778399] hover:text-[#1B2943] hover:bg-[#F4F6FF] transition-colors"
      onclick={() => open = false}
    >
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>

  <div class="px-5 pt-4 pb-1">
    <span class="label text-[10px] text-[#778399]">Navigation</span>
  </div>

  <nav class="flex-1 flex flex-col gap-1 px-3 py-1 overflow-y-auto">
    {#each navItems as item}
      <a
        href="#/{item.route}"
        onclick={handleNavClick}
        class="flex items-center gap-3 px-3 py-2 text-sm rounded-xl transition-all duration-150
          {isActive(item.route)
            ? 'bg-[#E8EEFF] text-[#2448CB] font-bold shadow-soft-sm'
            : 'text-[#64748B] hover:bg-[#F8FAFF] hover:text-[#1B2943]'}"
      >
        {#if item.icon === 'grid'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
        {:else if item.icon === 'message'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        {:else if item.icon === 'layers'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/></svg>
        {:else if item.icon === 'activity'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        {:else if item.icon === 'target'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>
        {:else if item.icon === 'dollar'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/></svg>
        {:else if item.icon === 'settings'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        {:else if item.icon === 'plug'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v6m-3-3h6m-9 5h12a2 2 0 012 2v7a2 2 0 01-2 2H6a2 2 0 01-2-2v-7a2 2 0 012-2z"/></svg>
        {:else if item.icon === 'shield'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
        {:else if item.icon === 'database'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
        {:else if item.icon === 'file'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
        {:else if item.icon === 'radio'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 010 8.49m-8.48-.01a6 6 0 010-8.49m11.31-2.82a10 10 0 010 14.14m-14.14 0a10 10 0 010-14.14"/></svg>
        {:else if item.icon === 'clock'}
          <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
        {/if}
        <span>{item.label}</span>
      </a>
    {/each}
  </nav>

  <div class="px-5 py-3.5 border-t border-[#DCE4F3] flex items-center justify-between bg-[#F8FAFF]">
    <div class="flex items-center gap-2 text-xs text-[#64748B]">
      <span
        class="w-2.5 h-2.5 rounded-full {status === 'connected'
          ? 'bg-[#219B70] shadow-sm animate-pulse-glow'
          : status === 'error' ? 'bg-[#D94B5B]' : 'bg-[#778399]'}"
      ></span>
      <span class="font-semibold text-xs text-[#1B2943]">{status === 'connected' ? 'Connected' : status === 'error' ? 'Error' : 'Offline'}</span>
    </div>
    <button
      type="button"
      onclick={handleLogout}
      class="text-xs font-semibold text-[#778399] hover:text-[#D94B5B] px-2 py-1 rounded-lg hover:bg-white transition-colors duration-150"
    >
      Disconnect
    </button>
  </div>
</aside>
