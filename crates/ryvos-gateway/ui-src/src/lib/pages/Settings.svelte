<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let healthData = $state(null);
  let metricsData = $state(null);
  let loading = $state(true);

  let budgetData = $state(null);
  let budgetLoading = $state(false);
  let budgetEditing = $state(false);
  let editBudgetDollars = $state('');
  let editWarnPct = $state('');
  let budgetSaving = $state(false);
  let budgetMessage = $state('');

  let modelData = $state(null);
  let availableModels = $state([]);
  let modelLoading = $state(false);
  let modelEditing = $state(false);
  let editModelId = $state('');
  let editTemperature = $state(0.7);
  let editMaxTokens = $state(4096);
  let editThinking = $state('off');
  let modelSaving = $state(false);
  let modelMessage = $state('');

  onMount(async () => {
    try {
      const [health, metrics] = await Promise.all([
        apiFetch('/api/health').catch(() => null),
        apiFetch('/api/metrics').catch(() => null),
      ]);
      healthData = health;
      metricsData = metrics;
    } finally {
      loading = false;
    }
    loadBudget();
    loadModel();
  });

  async function loadBudget() {
    budgetLoading = true;
    try {
      budgetData = await apiFetch('/api/budget');
      editBudgetDollars = budgetData.monthly_budget_cents ? (budgetData.monthly_budget_cents / 100).toFixed(2) : '';
      editWarnPct = budgetData.warn_pct != null ? String(budgetData.warn_pct) : '80';
    } catch {
      budgetData = null;
    } finally {
      budgetLoading = false;
    }
  }

  async function saveBudget() {
    budgetSaving = true;
    budgetMessage = '';
    try {
      const cents = Math.round(parseFloat(editBudgetDollars) * 100);
      const warnPct = parseInt(editWarnPct) || 80;
      await apiFetch('/api/budget', {
        method: 'PUT',
        body: JSON.stringify({ monthly_budget_cents: cents, warn_pct: warnPct }),
      });
      budgetMessage = 'Budget saved. Restart required for changes to take effect.';
      budgetEditing = false;
      await loadBudget();
    } catch (e) {
      budgetMessage = 'Error: ' + e.message;
    } finally {
      budgetSaving = false;
    }
  }

  async function loadModel() {
    modelLoading = true;
    try {
      const [current, available] = await Promise.all([
        apiFetch('/api/model').catch(() => null),
        apiFetch('/api/models/available').catch(() => ({ models: [] })),
      ]);
      modelData = current;
      availableModels = available.models || available || [];
      if (modelData) {
        editModelId = modelData.model_id || '';
        editTemperature = modelData.temperature != null ? modelData.temperature : 0.7;
        editMaxTokens = modelData.max_tokens || 4096;
        editThinking = modelData.thinking || 'off';
      }
    } catch {
      modelData = null;
    } finally {
      modelLoading = false;
    }
  }

  async function saveModel() {
    modelSaving = true;
    modelMessage = '';
    try {
      await apiFetch('/api/model', {
        method: 'PUT',
        body: JSON.stringify({
          model_id: editModelId,
          temperature: parseFloat(editTemperature),
          max_tokens: parseInt(editMaxTokens),
          thinking: editThinking,
        }),
      });
      modelMessage = 'Model saved. Restart required for changes to take effect.';
      modelEditing = false;
      await loadModel();
    } catch (e) {
      modelMessage = 'Error: ' + e.message;
    } finally {
      modelSaving = false;
    }
  }

  let budgetPct = $derived(metricsData && metricsData.monthly_budget_cents > 0
    ? Math.min(metricsData.budget_utilization_pct, 100) : 0);
  let budgetColor = $derived(budgetPct > 90 ? 'text-[#D94B5B]' : budgetPct > 70 ? 'text-[#C58A2A]' : 'text-[#2448CB]');
  let barColor = $derived(budgetPct > 90 ? 'bg-[#D94B5B]' : budgetPct > 70 ? 'bg-[#C58A2A]' : 'bg-[#2448CB]');
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Settings</h2>
    <p class="text-[#778399] text-sm mt-1">System runtime info, budget limits, and active model configuration</p>
  </div>

  {#if loading}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each Array(2) as _}
        <div class="card-soft p-6 min-h-[160px] animate-pulse"></div>
      {/each}
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="card-soft p-6">
        <h3 class="label mb-4 text-[#1B2943]">System Overview</h3>
        {#if healthData}
          <div class="space-y-4">
            <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
              <span class="label text-[10px]">Version</span>
              <p class="text-base font-bold text-[#1B2943] mt-0.5">{healthData.version || 'unknown'}</p>
            </div>
            <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
              <span class="label text-[10px]">Status</span>
              <p class="text-base font-bold text-[#219B70] uppercase mt-0.5">{healthData.status || 'unknown'}</p>
            </div>
          </div>
        {:else}
          <p class="text-[#778399] text-sm">Failed to load system info</p>
        {/if}
      </div>

      <div class="card-soft p-6">
        <h3 class="label mb-4 text-[#1B2943]">Budget Usage</h3>
        {#if metricsData && metricsData.monthly_budget_cents > 0}
          <div class="space-y-4">
            <div class="grid grid-cols-3 gap-2">
              <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
                <span class="label text-[10px]">Monthly</span>
                <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">${(metricsData.monthly_budget_cents / 100).toFixed(2)}</p>
              </div>
              <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
                <span class="label text-[10px]">Spent</span>
                <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">${(metricsData.total_cost_cents / 100).toFixed(2)}</p>
              </div>
              <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
                <span class="label text-[10px]">Utilized</span>
                <p class="text-base font-bold {budgetColor} tabular-nums mt-0.5">{metricsData.budget_utilization_pct}%</p>
              </div>
            </div>
            <div class="h-2.5 bg-[#EAF0FC] rounded-full overflow-hidden">
              <div class="{barColor} h-full rounded-full transition-all duration-500" style="width: {budgetPct}%"></div>
            </div>
          </div>
        {:else}
          <p class="text-[#778399] text-sm py-4">
            No budget configured. Add <code class="font-mono bg-[#E8EEFF] text-[#2448CB] px-1.5 py-0.5 rounded text-xs">[budget]</code> to your config.toml.
          </p>
        {/if}
      </div>
    </div>

    <div class="mt-4 card-soft p-6">
      <div class="flex items-center justify-between mb-4">
        <h3 class="label text-[#1B2943]">Budget Configuration</h3>
        {#if !budgetEditing}
          <button
            type="button"
            onclick={() => budgetEditing = true}
            class="btn-surface text-xs font-semibold px-3 py-1.5 hover:border-[#2448CB]"
          >
            Edit
          </button>
        {/if}
      </div>

      {#if budgetLoading}
        <p class="text-[#778399] text-sm animate-pulse">Loading budget...</p>
      {:else if budgetEditing}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label for="monthly-budget-input" class="label block mb-1">Monthly Budget ($)</label>
            <input
              id="monthly-budget-input"
              bind:value={editBudgetDollars}
              type="number"
              step="0.01"
              min="0"
              placeholder="10.00"
              class="input-soft w-full px-3 py-2 text-sm font-mono"
            />
          </div>
          <div>
            <label for="warn-pct-input" class="label block mb-1">Warn at (%)</label>
            <input
              id="warn-pct-input"
              bind:value={editWarnPct}
              type="number"
              min="0"
              max="100"
              placeholder="80"
              class="input-soft w-full px-3 py-2 text-sm font-mono"
            />
          </div>
        </div>
        <div class="flex items-center gap-3 mt-4">
          <button
            type="button"
            onclick={saveBudget}
            disabled={budgetSaving}
            class="btn-primary px-5 py-2 text-xs font-semibold disabled:opacity-40"
          >
            {budgetSaving ? 'Saving...' : 'Save Budget'}
          </button>
          <button
            type="button"
            onclick={() => budgetEditing = false}
            class="btn-surface px-4 py-2 text-xs font-semibold text-[#778399]"
          >
            Cancel
          </button>
        </div>
      {:else if budgetData}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Monthly Budget Limit</span>
            <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">
              {budgetData.monthly_budget_cents ? '$' + (budgetData.monthly_budget_cents / 100).toFixed(2) : 'Not configured'}
            </p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Warning Alert Threshold</span>
            <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">{budgetData.warn_pct != null ? budgetData.warn_pct + '%' : '80%'}</p>
          </div>
        </div>
      {:else}
        <p class="text-[#778399] text-sm">Budget endpoint not available</p>
      {/if}

      {#if budgetMessage}
        <div class="mt-4 px-4 py-3 bg-[#E8EEFF] border border-[#2448CB]/30 rounded-xl text-sm text-[#1B2943]">
          {budgetMessage}
        </div>
      {/if}
    </div>

    <div class="mt-4 card-soft p-6">
      <div class="flex items-center justify-between mb-4">
        <h3 class="label text-[#1B2943]">Model Configuration</h3>
        {#if !modelEditing}
          <button
            type="button"
            onclick={() => modelEditing = true}
            class="btn-surface text-xs font-semibold px-3 py-1.5 hover:border-[#2448CB]"
          >
            Edit
          </button>
        {/if}
      </div>

      {#if modelLoading}
        <p class="text-[#778399] text-sm animate-pulse">Loading model configuration...</p>
      {:else if modelEditing}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="md:col-span-2">
            <label for="select-model" class="label block mb-1">Model Selection</label>
            {#if availableModels.length > 0}
              <select
                id="select-model"
                bind:value={editModelId}
                class="input-soft w-full px-3 py-2 text-sm"
              >
                {#each availableModels as model}
                  <option value={typeof model === 'string' ? model : model.id}>
                    {typeof model === 'string' ? model : (model.name || model.id)}
                  </option>
                {/each}
              </select>
            {:else}
              <input
                id="select-model"
                bind:value={editModelId}
                placeholder="claude-sonnet-4-20250514"
                class="input-soft w-full px-3 py-2 text-sm font-mono"
              />
            {/if}
          </div>
          <div>
            <label for="temperature-range" class="label block mb-1">Temperature ({editTemperature})</label>
            <input
              id="temperature-range"
              bind:value={editTemperature}
              type="range"
              min="0"
              max="1"
              step="0.1"
              class="w-full accent-[#2448CB]"
            />
            <div class="flex justify-between text-[10px] text-[#778399] mt-1 tabular-nums">
              <span>0.0 (Precise)</span>
              <span>1.0 (Creative)</span>
            </div>
          </div>
          <div>
            <label for="max-tokens-input" class="label block mb-1">Max Tokens</label>
            <input
              id="max-tokens-input"
              bind:value={editMaxTokens}
              type="number"
              min="1"
              max="200000"
              placeholder="4096"
              class="input-soft w-full px-3 py-2 text-sm font-mono"
            />
          </div>
          <div>
            <label for="thinking-select" class="label block mb-1">Thinking Mode</label>
            <select
              id="thinking-select"
              bind:value={editThinking}
              class="input-soft w-full px-3 py-2 text-sm"
            >
              <option value="off">Off</option>
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
          </div>
        </div>
        <div class="flex items-center gap-3 mt-4">
          <button
            type="button"
            onclick={saveModel}
            disabled={modelSaving}
            class="btn-primary px-5 py-2 text-xs font-semibold disabled:opacity-40"
          >
            {modelSaving ? 'Saving...' : 'Save Model'}
          </button>
          <button
            type="button"
            onclick={() => modelEditing = false}
            class="btn-surface px-4 py-2 text-xs font-semibold text-[#778399]"
          >
            Cancel
          </button>
        </div>
      {:else if modelData}
        <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Provider</span>
            <p class="text-base font-bold text-[#1B2943] mt-0.5">{modelData.provider || 'unknown'}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3] col-span-2 md:col-span-1">
            <span class="label text-[10px]">Model ID</span>
            <p class="text-sm font-bold font-mono text-[#1B2943] truncate mt-0.5">{modelData.model_id || 'unknown'}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Temperature</span>
            <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">{modelData.temperature != null ? modelData.temperature : '-'}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Max Tokens</span>
            <p class="text-base font-bold text-[#1B2943] tabular-nums mt-0.5">{modelData.max_tokens || '-'}</p>
          </div>
          <div class="p-3 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
            <span class="label text-[10px]">Thinking</span>
            <p class="text-base font-bold text-[#1B2943] capitalize mt-0.5">{modelData.thinking || 'off'}</p>
          </div>
        </div>
      {:else}
        <p class="text-[#778399] text-sm">Model endpoint not available</p>
      {/if}

      {#if modelMessage}
        <div class="mt-4 px-4 py-3 bg-[#E8EEFF] border border-[#2448CB]/30 rounded-xl text-sm text-[#1B2943]">
          {modelMessage}
        </div>
      {/if}
    </div>
  {/if}
</div>
