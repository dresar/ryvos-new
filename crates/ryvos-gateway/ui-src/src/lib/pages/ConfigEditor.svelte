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
      error = e.message.includes('403') ? 'admin_required' : e.message;
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
      saveMessage = 'Configuration saved successfully';
      originalContent = configContent;
      setTimeout(() => { saveMessage = ''; }, 3000);
    } catch (e) {
      saveMessage = 'Error saving: ' + e.message;
    } finally {
      saving = false;
    }
  }

  let hasChanges = $derived(configContent !== originalContent);
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Configuration</h2>
    <p class="text-[#778399] text-sm mt-1">Directly inspect and modify runtime config.toml</p>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading configuration...</p>
    </div>
  {:else if error === 'admin_required'}
    <div class="card-soft p-12 text-center">
      <div class="w-12 h-12 rounded-2xl bg-[#FFFBEB] text-[#C58A2A] flex items-center justify-center mx-auto mb-3">
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      </div>
      <p class="text-[#1B2943] text-sm font-semibold mb-1">Admin Access Required</p>
      <p class="text-[#778399] text-xs max-w-sm mx-auto">The configuration editor requires an Admin API key. Connect with an admin key to edit config.toml.</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Configuration endpoint not available: {error}</p>
    </div>
  {:else}
    <div class="card-soft p-5">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-sm font-bold text-[#1B2943] font-mono">config.toml</h3>
        <div class="flex items-center gap-3">
          {#if saveMessage}
            <span class="text-xs font-medium {saveMessage.startsWith('Error') ? 'text-[#D94B5B]' : 'text-[#219B70]'}">
              {saveMessage}
            </span>
          {/if}
          {#if hasChanges}
            <span class="text-xs font-medium text-[#C58A2A]">Unsaved changes</span>
          {/if}
          <button
            type="button"
            onclick={saveConfig}
            disabled={saving || !hasChanges}
            class="btn-primary px-4 py-1.5 text-xs disabled:opacity-40"
          >
            {saving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
      <textarea
        bind:value={configContent}
        spellcheck="false"
        class="input-soft w-full h-[500px] p-4 font-mono text-[#1B2943] text-xs leading-relaxed resize-y"
      ></textarea>
    </div>
  {/if}
</div>
