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
    gmail: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Google Cloud Console > APIs and Services > Credentials > Create OAuth Client ID' },
    calendar: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Google Cloud Console: OAuth covers Calendar and Drive' },
    drive: { url: 'https://console.cloud.google.com/apis/credentials', label: 'Google Cloud Console: OAuth covers Calendar and Drive' },
    slack: { url: 'https://api.slack.com/apps', label: 'Slack API > Your Apps > Create New App > OAuth and Permissions' },
    github: { url: 'https://github.com/settings/developers', label: 'GitHub > Settings > Developer Settings > OAuth Apps > New' },
    jira: { url: 'https://developer.atlassian.com/console/myapps/', label: 'Atlassian Developer > My Apps > Create > OAuth 2.0' },
    linear: { url: 'https://linear.app/settings/api', label: 'Linear > Settings > API > Create OAuth Application' },
    notion: { url: 'https://www.notion.so/my-integrations', label: 'Notion > My Integrations > Create New Integration > Copy Internal Token' },
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
      error = e.message || 'Failed to load integrations';
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
        content += '\n\n[integrations.notion]\napi_key = "' + setupForm.api_key + '"\n';
      } else {
        if (!setupForm.client_id || !setupForm.client_secret) { setupError = 'Both client ID and secret are required'; setupSaving = false; return; }
        if (content.includes('[integrations.' + configKey + ']')) {
          setupError = '[integrations.' + configKey + '] already exists in config.toml. Edit via Config page.';
          setupSaving = false;
          return;
        }
        content += '\n\n[integrations.' + configKey + ']\nclient_id = "' + setupForm.client_id + '"\nclient_secret = "' + setupForm.client_secret + '"\n';
      }

      const res = await apiFetch('/api/config', {
        method: 'PUT',
        body: JSON.stringify({ content }),
      });
      if (res.error) {
        setupError = res.error;
      } else {
        successApp = app.name + ' configured. Restart the daemon, then click Connect.';
        setTimeout(() => { successApp = ''; }, 6000);
        closeSetup();
        await loadApps();
      }
    } catch (e) {
      setupError = e.message.includes('403') ? 'Admin access required to save configuration' : e.message;
    } finally {
      setupSaving = false;
    }
  }

  async function connectApp(app) {
    actionInProgress = app.id;
    try {
      const data = await apiFetch('/api/integrations/' + app.id + '/connect', { method: 'POST' });
      if (data.error) {
        error = data.error;
      } else if (data.redirect_url) {
        window.open(data.redirect_url, '_blank', 'width=600,height=700');
      } else if (data.connected) {
        successApp = app.name + ' successfully connected';
        setTimeout(() => { successApp = ''; }, 4000);
        await loadApps();
      }
    } catch (e) {
      error = 'Failed to connect ' + app.name + ': ' + e.message;
    } finally {
      actionInProgress = '';
    }
  }

  async function disconnectApp(app) {
    actionInProgress = app.id;
    try {
      await apiFetch('/api/integrations/' + app.id, { method: 'DELETE' });
      await loadApps();
    } catch (e) {
      error = 'Failed to disconnect ' + app.name + ': ' + e.message;
    } finally {
      actionInProgress = '';
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">External Integrations</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Connect SaaS platforms, OAuth credentials, and workspace services</p>
  </div>

  {#if successApp}
    <div class="p-4 rounded-xl bg-green-500/10 border border-green-500/20 text-sm text-[var(--color-text-main)] flex items-center justify-between">
      <span>{successApp}</span>
      <button type="button" onclick={() => { successApp = ''; }} class="text-xs font-semibold uppercase text-[var(--color-text-muted)]">Dismiss</button>
    </div>
  {/if}

  {#if error}
    <div class="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-sm text-[var(--color-danger)] flex items-center justify-between">
      <span>{error}</span>
      <button type="button" onclick={() => { error = ''; }} class="text-xs font-semibold uppercase text-[var(--color-text-muted)]">Dismiss</button>
    </div>
  {/if}

  {#if loading}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      {#each Array(4) as _}
        <div class="neu-card p-6 min-h-[140px] animate-pulse"></div>
      {/each}
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      {#each apps as app (app.id)}
        <div class="neu-card p-6 space-y-4">
          <div class="flex items-start justify-between">
            <div>
              <h3 class="text-base font-bold text-[var(--color-text-main)]">{app.name}</h3>
              <p class="text-xs text-[var(--color-text-muted)] mt-0.5">{app.actions} action{app.actions !== 1 ? 's' : ''} available</p>
            </div>
            {#if app.connected}
              <span class="inline-flex px-2.5 py-1 text-xs font-semibold rounded-full badge-success">
                Connected
              </span>
            {:else if app.configured}
              <span class="inline-flex px-2.5 py-1 text-xs font-semibold rounded-full badge-info">
                Configured
              </span>
            {:else}
              <span class="inline-flex px-2.5 py-1 text-xs font-semibold rounded-full badge-neutral">
                Unconfigured
              </span>
            {/if}
          </div>

          {#if app.connected}
            <button
              type="button"
              onclick={() => disconnectApp(app)}
              disabled={actionInProgress === app.id}
              class="btn-danger min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl disabled:opacity-40"
            >
              {actionInProgress === app.id ? 'Disconnecting...' : 'Disconnect Service'}
            </button>
          {:else if app.configured}
            <button
              type="button"
              onclick={() => connectApp(app)}
              disabled={actionInProgress === app.id}
              class="btn-primary min-h-[44px] px-5 py-2 text-xs font-semibold rounded-xl disabled:opacity-40"
            >
              {actionInProgress === app.id ? 'Authorizing...' : 'Connect OAuth'}
            </button>
          {:else if setupApp === app.id}
            <div class="pt-4 border-t border-[var(--color-border)] space-y-3">
              {#if setupLinks[app.id]}
                <a
                  href={setupLinks[app.id].url}
                  target="_blank"
                  rel="noopener"
                  class="text-xs text-[var(--color-primary)] hover:underline block leading-relaxed"
                >
                  {setupLinks[app.id].label}
                </a>
              {/if}

              {#if app.id === 'notion'}
                <div class="space-y-1">
                  <label for="setup-api-key" class="text-xs font-semibold text-[var(--color-text-muted)]">
                    Internal Integration Secret
                  </label>
                  <input
                    id="setup-api-key"
                    bind:value={setupForm.api_key}
                    type="password"
                    placeholder="secret_..."
                    class="neu-inset min-h-[44px] w-full px-3 py-2 text-xs font-mono rounded-xl text-[var(--color-text-main)]"
                  />
                </div>
              {:else}
                <div class="space-y-1">
                  <label for="setup-client-id" class="text-xs font-semibold text-[var(--color-text-muted)]">
                    OAuth Client ID
                  </label>
                  <input
                    id="setup-client-id"
                    bind:value={setupForm.client_id}
                    type="text"
                    placeholder="Client identifier..."
                    class="neu-inset min-h-[44px] w-full px-3 py-2 text-xs font-mono rounded-xl text-[var(--color-text-main)]"
                  />
                </div>
                <div class="space-y-1">
                  <label for="setup-client-secret" class="text-xs font-semibold text-[var(--color-text-muted)]">
                    OAuth Client Secret
                  </label>
                  <input
                    id="setup-client-secret"
                    bind:value={setupForm.client_secret}
                    type="password"
                    placeholder="Client secret..."
                    class="neu-inset min-h-[44px] w-full px-3 py-2 text-xs font-mono rounded-xl text-[var(--color-text-main)]"
                  />
                </div>
              {/if}

              {#if setupError}
                <p class="text-xs text-[var(--color-danger)]">{setupError}</p>
              {/if}

              <div class="flex items-center gap-2 pt-2">
                <button
                  type="button"
                  onclick={() => saveSetup(app)}
                  disabled={setupSaving}
                  class="btn-primary min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl disabled:opacity-40"
                >
                  {setupSaving ? 'Saving...' : 'Save Configuration'}
                </button>
                <button
                  type="button"
                  onclick={closeSetup}
                  class="btn-soft min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl text-[var(--color-text-muted)]"
                >
                  Cancel
                </button>
              </div>
            </div>
          {:else}
            <button
              type="button"
              onclick={() => openSetup(app)}
              class="btn-soft min-h-[44px] px-5 py-2 text-xs font-semibold rounded-xl text-[var(--color-text-main)]"
            >
              Configure Credentials
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>