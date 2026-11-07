<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let configContent = $state('');
  let loading = $state(true);
  let error = $state('');
  let saving = $state(false);
  let saveMessage = $state('');
  let originalContent = $state('');

  onMount(async () => {
    try {
      const data = await apiFetch('/api/config');
      configContent = data.content || data.config || (typeof data === 'string' ? data : JSON.stringify(data, null, 2));
      originalContent = configContent;
    } catch (e) {
      error = e.message.includes('403') ? 'admin_required' : (e.message || 'Failed to load configuration');
    } finally {
      loading = false;
    }
  });

  async function saveConfig() {
    saving = true;
    saveMessage = '';
    try {
      await apiFetch('/api/config', {
        method: 'PUT',
        body: JSON.stringify({ content: configContent }),
      });
      saveMessage = 'Configuration updated successfully';
      originalContent = configContent;
      setTimeout(() => { saveMessage = ''; }, 3000);
    } catch (e) {
      saveMessage = 'Error saving configuration: ' + (e.message || 'Network failure');
    } finally {
      saving = false;
    }
  }

  let hasChanges = $derived(configContent !== originalContent);
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">System Configuration</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Directly inspect and modify runtime settings in config.toml</p>
  </div>

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading configuration payload...</p>
    </div>
  {:else if error === 'admin_required'}
    <div class="neu-card p-12 text-center border-l-4 border-[var(--color-warning)]">
      <h3 class="text-base font-bold text-[var(--color-text-main)] mb-1">Administrator Privileges Required</h3>
      <p class="text-xs text-[var(--color-text-muted)] max-w-sm mx-auto">Editing config.toml requires an administrator authorization token. Connect with an admin API key to unlock changes.</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Configuration Error</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else}
    <div class="neu-card p-6 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <span class="font-mono text-xs font-bold text-[var(--color-primary)]">config.toml</span>
          {#if hasChanges}
            <span class="inline-flex px-2 py-0.5 rounded-full text-xs font-semibold badge-warning">
              Unsaved Changes
            </span>
          {/if}
        </div>
        <div class="flex items-center gap-3">
          {#if saveMessage}
            <span class="text-xs font-semibold {saveMessage.startsWith('Error') ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}">
              {saveMessage}
            </span>
          {/if}
          <button
            type="button"
            onclick={saveConfig}
            disabled={saving || !hasChanges}
            class="btn-primary min-h-[44px] px-5 py-2 text-xs font-semibold rounded-xl disabled:opacity-40"
          >
            {saving ? 'Writing Config...' : 'Save Configuration'}
          </button>
        </div>
      </div>
      <textarea
        bind:value={configContent}
        spellcheck="false"
        class="neu-inset w-full h-[520px] p-4 font-mono text-xs text-[var(--color-text-main)] leading-relaxed rounded-xl resize-y"
      ></textarea>
    </div>
  {/if}
</div>