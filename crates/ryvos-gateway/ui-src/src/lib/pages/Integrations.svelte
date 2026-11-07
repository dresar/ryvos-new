<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let apps = $state([]);
  let loading = $state(true);
  let error = $state('');
  let successApp = $state('');
  let actionInProgress = $state('');
  let setupApp = $state(null);
  let setupForm = $state({ client_id: '', client_secret: '', api_key: '' });
  let setupSaving = $state(false);
  let setupError = $state('');

  const setupLinks = {
    gmail: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Google Cloud Console -> APIs & Services -> Credentials -> Create OAuth Client ID' },
    calendar: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Same as Gmail: Google OAuth covers Calendar + Drive' },
    drive: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Same as Gmail: Google OAuth covers Calendar + Drive' },
    slack: { url: 'https://api.slack.com/apps', label: 'Slack API -> Your Apps -> Create New App -> OAuth & Permissions' },
    github: { url: 'https://github.com/settings/developers', label: 'GitHub -> Settings -> Developer Settings -> OAuth Apps -> New' },
    jira: { url: 'https://developer.atlassian.com/console/myapps/', label: 'Atlassian Developer -> My Apps -> Create -> OAuth 2.0' },
    linear: { url: 'https://linear.app/settings/api', label: 'Linear -> Settings -> API -> Create OAuth Application' },
    notion: { url: 'https://www.notion.so/my-integrations', label: 'Notion -> My Integrations -> Create New Integration -> Copy Internal Token' },
  };

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    const connected = params.get('connected');
    if (connected) {
      successApp = connected;
      const url = new URL(window.location);
      url.searchParams.delete('connected');
      window.history.replaceState({}, '', url);
      setTimeout(() => { successApp = ''; }, 4000);
    }
    await loadApps();
  });

  async function loadApps() {
    loading = true;
    error = '';
    try {
      const data = await apiFetch('/api/integrations');
      apps = data.apps || [];
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  function openSetup(app) {
    setupApp = app.id;
    setupForm = { client_id: '', client_secret: '', api_key: '' };
    setupError = '';
  }

  function closeSetup() {
    setupApp = null;
    setupForm = { client_id: '', client_secret: '', api_key: '' };
    setupError = '';
  }

  async function saveSetup(app) {
    setupSaving = true;
    setupError = '';
    try {
      const configData = await apiFetch('/api/config');
      let content = configData.content || '';

      const appId = app.id;
      const configKey = (appId === 'calendar' || appId === 'drive') ? 'gmail' : appId;

      if (configKey === 'notion') {
        if (!setupForm.api_key) { setupError = 'API key is required'; setupSaving = false; return; }
        content += `\n\n[integrations.notion]\napi_key = "${setupForm.api_key}"\n`;
      } else {
        if (!setupForm.client_id || !setupForm.client_secret) { setupError = 'Both fields are required'; setupSaving = false; return; }
        if (content.includes(`[integrations.${configKey}]`)) {
          setupError = `[integrations.${configKey}] already exists in config.toml. Edit it via the Config page.`;
          setupSaving = false;
          return;
        }
        content += `\n\n[integrations.${configKey}]\nclient_id = "${setupForm.client_id}"\nclient_secret = "${setupForm.client_secret}"\n`;
      }

      const res = await apiFetch('/api/config', {
        method: 'PUT',
        body: JSON.stringify({ content }),
      });
      if (res.error) {
        setupError = res.error;
      } else {
        successApp = `${app.name} configured! Restart the daemon, then click Connect.`;
        setTimeout(() => { successApp = ''; }, 6000);
        closeSetup();
        await loadApps();
      }
    } catch (e) {
      setupError = e.message.includes('403') ? 'Admin access required to save config' : e.message;
    } finally {
      setupSaving = false;
    }
  }

  async function connectApp(app) {
    actionInProgress = app.id;
    try {
      const data = await apiFetch(`/api/integrations/${app.id}/connect`, { method: 'POST' });
      if (data.error) {
        error = data.error;
      } else if (data.redirect_url) {
        window.open(data.redirect_url, '_blank', 'width=600,height=700');
      } else if (data.connected) {
        successApp = app.name;
        setTimeout(() => { successApp = ''; }, 4000);
        await loadApps();
      }
    } catch (e) {
      error = `Failed to connect ${app.name}: ${e.message}`;
    } finally {
      actionInProgress = '';
    }
  }

  async function disconnectApp(app) {
    actionInProgress = app.id;
    try {
      await apiFetch(`/api/integrations/${app.id}`, { method: 'DELETE' });
      await loadApps();
    } catch (e) {
      error = `Failed to disconnect ${app.name}: ${e.message}`;
    } finally {
      actionInProgress = '';
    }
  }
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Integrations</h2>
    <p class="text-[#778399] text-sm mt-1">Connect external services and OAuth tools to your autonomous agent</p>
  </div>

  {#if successApp}
    <div class="mb-5 px-4 py-3 bg-[#219B70]/10 border border-[#219B70] rounded-xl text-sm text-[#1B2943] flex items-center justify-between">
      <span>{successApp}</span>
      <button type="button" onclick={() => successApp = ''} class="text-[#778399] hover:text-[#1B2943] text-xs font-semibold uppercase">Dismiss</button>
    </div>
  {/if}

  {#if error}
    <div class="mb-5 px-4 py-3 bg-[#D94B5B]/10 border border-[#D94B5B] rounded-xl text-sm text-[#1B2943] flex items-center justify-between">
      <span>{error}</span>
      <button type="button" onclick={() => error = ''} class="text-[#778399] hover:text-[#1B2943] text-xs font-semibold uppercase">Dismiss</button>
    </div>
  {/if}

  {#if loading}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each Array(4) as _}
        <div class="card-soft p-5 min-h-[120px] animate-pulse"></div>
      {/each}
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each apps as app (app.id)}
        <div class="card-soft p-5 transition-all duration-200">
          <div class="flex items-start justify-between mb-3">
            <div>
              <h3 class="text-base font-bold text-[#1B2943]">{app.name}</h3>
              <p class="text-xs text-[#778399] mt-0.5">{app.actions} action{app.actions !== 1 ? 's' : ''}</p>
            </div>
            {#if app.connected}
              <span class="bg-[#219B70]/10 text-[#219B70] border border-[#219B70]/30 text-xs font-semibold px-2.5 py-0.5 rounded-full">Connected</span>
            {:else if app.configured}
              <span class="bg-[#E8EEFF] text-[#2448CB] border border-[#2448CB]/30 text-xs font-semibold px-2.5 py-0.5 rounded-full">Ready</span>
            {/if}
          </div>

          {#if app.connected}
            <button
              type="button"
              onclick={() => disconnectApp(app)}
              disabled={actionInProgress === app.id}
              class="btn-surface text-xs font-semibold px-3.5 py-1.5 text-[#778399] hover:text-[#D94B5B] hover:border-[#D94B5B] disabled:opacity-40"
            >
              {actionInProgress === app.id ? 'Disconnecting...' : 'Disconnect'}
            </button>
          {:else if app.configured}
            <button
              type="button"
              onclick={() => connectApp(app)}
              disabled={actionInProgress === app.id}
              class="btn-primary text-xs font-semibold px-4 py-2 disabled:opacity-40"
            >
              {actionInProgress === app.id ? 'Connecting...' : 'Connect'}
            </button>
          {:else if setupApp === app.id}
            <div class="mt-3 border-t border-[#DCE4F3] pt-3">
              {#if setupLinks[app.id]}
                <a href={setupLinks[app.id].url} target="_blank" rel="noopener"
                   class="text-xs text-[#2448CB] hover:underline mb-3 block">
                  {setupLinks[app.id].label}
                </a>
              {/if}

              {#if app.id === 'notion'}
                <div class="mb-3">
                  <label for="setup-api-key" class="label block mb-1">Internal Integration Token</label>
                  <input
                    id="setup-api-key"
                    bind:value={setupForm.api_key}
                    type="password"
                    placeholder="ntn_..."
                    class="input-soft w-full text-xs px-3 py-2 font-mono"
                  />
                </div>
              {:else}
                <div class="mb-2">
                  <label for="setup-client-id" class="label block mb-1">Client ID</label>
                  <input
                    id="setup-client-id"
                    bind:value={setupForm.client_id}
                    type="text"
                    placeholder="xxxx.apps.googleusercontent.com"
                    class="input-soft w-full text-xs px-3 py-2 font-mono"
                  />
                </div>
                <div class="mb-3">
                  <label for="setup-client-secret" class="label block mb-1">Client Secret</label>
                  <input
                    id="setup-client-secret"
                    bind:value={setupForm.client_secret}
                    type="password"
                    placeholder="GOCSPX-..."
                    class="input-soft w-full text-xs px-3 py-2 font-mono"
                  />
                </div>
              {/if}

              {#if setupError}
                <p class="text-xs text-[#D94B5B] mb-2">{setupError}</p>
              {/if}

              <div class="flex gap-2">
                <button
                  type="button"
                  onclick={() => saveSetup(app)}
                  disabled={setupSaving}
                  class="btn-primary text-xs px-4 py-2 disabled:opacity-40"
                >
                  {setupSaving ? 'Saving...' : 'Save & Configure'}
                </button>
                <button
                  type="button"
                  onclick={closeSetup}
                  class="btn-surface text-xs px-3 py-1.5 text-[#778399]"
                >
                  Cancel
                </button>
              </div>
            </div>
          {:else}
            <button
              type="button"
              onclick={() => openSetup(app)}
              class="btn-surface text-xs font-semibold px-4 py-2 hover:border-[#2448CB]"
            >
              Setup
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
