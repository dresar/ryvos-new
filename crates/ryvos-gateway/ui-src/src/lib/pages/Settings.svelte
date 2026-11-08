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

  let skillsList = $state([]);
  let skillsLoading = $state(false);

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
    loadSkills();
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
      budgetMessage = 'Budget parameters saved. Service restart required for new limits.';
      budgetEditing = false;
      await loadBudget();
    } catch (e) {
      budgetMessage = 'Error: ' + (e.message || 'Failed to save budget');
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
      const m = current?.model || current;
      modelData = m;
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
      modelMessage = 'Model configuration updated. Service restart required for changes to activate.';
      modelEditing = false;
      await loadModel();
    } catch (e) {
      modelMessage = 'Error: ' + (e.message || 'Failed to save model settings');
    } finally {
      modelSaving = false;
    }
  }

  async function loadSkills() {
    skillsLoading = true;
    try {
      const data = await apiFetch('/api/skills');
      skillsList = data.skills || [];
    } catch {
      skillsList = [];
    } finally {
      skillsLoading = false;
    }
  }

  let budgetPct = $derived(metricsData && metricsData.monthly_budget_cents > 0
    ? Math.min(metricsData.budget_utilization_pct || 0, 100) : 0);
  let budgetColor = $derived(budgetPct > 90 ? 'text-[var(--color-danger)]' : budgetPct > 70 ? 'text-[var(--color-warning)]' : 'text-[var(--color-primary)]');
  let barColor = $derived(budgetPct > 90 ? 'bg-[var(--color-danger)]' : budgetPct > 70 ? 'bg-[var(--color-warning)]' : 'bg-[var(--color-primary)]');
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">System Settings and Diagnostics</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Runtime infrastructure telemetry, LLM provider routing, budget constraints, and active skills</p>
  </div>

  {#if loading}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6">
      <div class="neu-card p-4 sm:p-6 min-h-[160px] animate-pulse"></div>
      <div class="neu-card p-4 sm:p-6 min-h-[160px] animate-pulse"></div>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6">
      <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">Core Service State</h3>
        {#if healthData}
          <div class="grid grid-cols-2 gap-2.5 sm:gap-4">
            <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
              <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Ryvos Version</span>
              <p class="text-sm sm:text-base font-bold text-[var(--color-text-main)] mt-1 font-mono">{healthData.version || '0.1.0'}</p>
            </div>
            <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
              <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Gateway Daemon</span>
              <p class="text-sm sm:text-base font-bold text-[var(--color-success)] mt-1 uppercase">{healthData.status || 'Active'}</p>
            </div>
          </div>
        {:else}
          <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">Core diagnostics unavailable</p>
        {/if}
      </div>

      <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">Budget Consumption</h3>
        {#if metricsData && metricsData.monthly_budget_cents > 0}
          <div class="space-y-3">
            <div class="grid grid-cols-3 gap-2 sm:gap-3">
              <div class="neu-inset p-2 sm:p-3 rounded-xl text-center">
                <span class="text-[10px] text-[var(--color-text-muted)] block">Cap</span>
                <p class="text-xs sm:text-sm font-bold text-[var(--color-text-main)] tabular-nums mt-0.5">${(metricsData.monthly_budget_cents / 100).toFixed(2)}</p>
              </div>
              <div class="neu-inset p-2 sm:p-3 rounded-xl text-center">
                <span class="text-[10px] text-[var(--color-text-muted)] block">Used</span>
                <p class="text-xs sm:text-sm font-bold text-[var(--color-text-main)] tabular-nums mt-0.5">${(metricsData.total_cost_cents / 100).toFixed(2)}</p>
              </div>
              <div class="neu-inset p-2 sm:p-3 rounded-xl text-center">
                <span class="text-[10px] text-[var(--color-text-muted)] block">Ratio</span>
                <p class="text-xs sm:text-sm font-bold {budgetColor} tabular-nums mt-0.5">{metricsData.budget_utilization_pct}%</p>
              </div>
            </div>
            <div class="h-2 bg-[var(--color-border)] rounded-full overflow-hidden">
              <div class="{barColor} h-full rounded-full transition-all duration-500" style="width: {budgetPct}%"></div>
            </div>
          </div>
        {:else}
          <p class="text-xs sm:text-sm text-[var(--color-text-muted)] py-2">
            No budget ceiling enforced. Configure [budget] section in config.toml to cap spending.
          </p>
        {/if}
      </div>
    </div>

    <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">Budget Allocation Controls</h3>
        {#if !budgetEditing}
          <button
            type="button"
            onclick={() => { budgetEditing = true; }}
            class="btn-soft min-h-[36px] px-3 py-1.5 text-xs font-semibold rounded-xl text-[var(--color-primary)]"
          >
            Modify Budget
          </button>
        {/if}
      </div>

      {#if budgetLoading}
        <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">Fetching budget parameters...</p>
      {:else if budgetEditing}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
          <div class="space-y-1">
            <label for="monthly-budget-input" class="text-xs font-semibold text-[var(--color-text-muted)]">Monthly Spend Cap (USD)</label>
            <input
              id="monthly-budget-input"
              bind:value={editBudgetDollars}
              type="number"
              step="0.01"
              min="0"
              placeholder="10.00"
              class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm font-mono text-[var(--color-text-main)] rounded-xl"
            />
          </div>
          <div class="space-y-1">
            <label for="warn-pct-input" class="text-xs font-semibold text-[var(--color-text-muted)]">Warning Alert Threshold (%)</label>
            <input
              id="warn-pct-input"
              bind:value={editWarnPct}
              type="number"
              min="0"
              max="100"
              placeholder="80"
              class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm font-mono text-[var(--color-text-main)] rounded-xl"
            />
          </div>
        </div>
        <div class="flex items-center gap-3 pt-2">
          <button
            type="button"
            onclick={saveBudget}
            disabled={budgetSaving}
            class="btn-primary min-h-[36px] px-4 py-1.5 text-xs font-semibold rounded-xl disabled:opacity-40"
          >
            {budgetSaving ? 'Saving...' : 'Apply Budget'}
          </button>
          <button
            type="button"
            onclick={() => { budgetEditing = false; }}
            class="btn-soft min-h-[36px] px-3 py-1.5 text-xs font-semibold rounded-xl text-[var(--color-text-muted)]"
          >
            Cancel
          </button>
        </div>
      {:else if budgetData}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Allocated Monthly Limit</span>
            <p class="text-sm sm:text-base font-bold text-[var(--color-text-main)] tabular-nums mt-1 font-mono">
              {budgetData.monthly_budget_cents ? '$' + (budgetData.monthly_budget_cents / 100).toFixed(2) : 'Unrestricted'}
            </p>
          </div>
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Notification Alert Level</span>
            <p class="text-sm sm:text-base font-bold text-[var(--color-text-main)] tabular-nums mt-1 font-mono">
              {budgetData.warn_pct != null ? budgetData.warn_pct + '%' : '80%'}
            </p>
          </div>
        </div>
      {/if}

      {#if budgetMessage}
        <div class="p-2.5 rounded-xl bg-blue-500/10 text-xs font-semibold text-[var(--color-text-main)]">
          {budgetMessage}
        </div>
      {/if}
    </div>

    <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">Active Model and LLM Parameters</h3>
        {#if !modelEditing}
          <button
            type="button"
            onclick={() => { modelEditing = true; }}
            class="btn-soft min-h-[36px] px-3 py-1.5 text-xs font-semibold rounded-xl text-[var(--color-primary)]"
          >
            Modify Model
          </button>
        {/if}
      </div>

      {#if modelLoading}
        <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">Loading model definitions...</p>
      {:else if modelEditing}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
          <div class="md:col-span-2 space-y-1">
            <label for="select-model" class="text-xs font-semibold text-[var(--color-text-muted)]">Model ID</label>
            {#if availableModels.length > 0}
              <select
                id="select-model"
                bind:value={editModelId}
                class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm text-[var(--color-text-main)] rounded-xl bg-[var(--color-surface)]"
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
                placeholder="anthropic/claude-sonnet-4-20250514"
                class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm font-mono text-[var(--color-text-main)] rounded-xl"
              />
            {/if}
          </div>
          <div class="space-y-1">
            <label for="temperature-range" class="text-xs font-semibold text-[var(--color-text-muted)]">Temperature ({editTemperature})</label>
            <input
              id="temperature-range"
              bind:value={editTemperature}
              type="range"
              min="0"
              max="1"
              step="0.05"
              class="w-full accent-[var(--color-primary)]"
            />
            <div class="flex justify-between text-[10px] text-[var(--color-text-muted)] tabular-nums">
              <span>0.0 (Deterministic)</span>
              <span>1.0 (Creative)</span>
            </div>
          </div>
          <div class="space-y-1">
            <label for="max-tokens-input" class="text-xs font-semibold text-[var(--color-text-muted)]">Max Tokens</label>
            <input
              id="max-tokens-input"
              bind:value={editMaxTokens}
              type="number"
              min="1"
              max="200000"
              placeholder="4096"
              class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm font-mono text-[var(--color-text-main)] rounded-xl"
            />
          </div>
          <div class="space-y-1">
            <label for="thinking-select" class="text-xs font-semibold text-[var(--color-text-muted)]">Extended Thinking</label>
            <select
              id="thinking-select"
              bind:value={editThinking}
              class="neu-inset min-h-[36px] w-full px-2.5 py-1.5 text-xs sm:text-sm text-[var(--color-text-main)] rounded-xl bg-[var(--color-surface)]"
            >
              <option value="off">Off</option>
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
          </div>
        </div>
        <div class="flex items-center gap-3 pt-2">
          <button
            type="button"
            onclick={saveModel}
            disabled={modelSaving}
            class="btn-primary min-h-[36px] px-4 py-1.5 text-xs font-semibold rounded-xl disabled:opacity-40"
          >
            {modelSaving ? 'Saving...' : 'Apply Model'}
          </button>
          <button
            type="button"
            onclick={() => { modelEditing = false; }}
            class="btn-soft min-h-[36px] px-3 py-1.5 text-xs font-semibold rounded-xl text-[var(--color-text-muted)]"
          >
            Cancel
          </button>
        </div>
      {:else if modelData}
        <div class="grid grid-cols-2 md:grid-cols-4 gap-2.5 sm:gap-4">
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Provider</span>
            <p class="text-xs sm:text-sm font-bold text-[var(--color-text-main)] mt-1 uppercase">{modelData.provider || 'default'}</p>
          </div>
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl md:col-span-1 col-span-2">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Model Identifier</span>
            <p class="text-xs font-bold font-mono text-[var(--color-primary)] mt-1 truncate" title={modelData.model_id}>{modelData.model_id || 'default'}</p>
          </div>
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Temperature</span>
            <p class="text-xs sm:text-sm font-bold text-[var(--color-text-main)] mt-1 tabular-nums">{modelData.temperature != null ? modelData.temperature : 0.7}</p>
          </div>
          <div class="neu-inset p-2.5 sm:p-4 rounded-xl">
            <span class="text-[10px] sm:text-xs text-[var(--color-text-muted)] block">Thinking Mode</span>
            <p class="text-xs sm:text-sm font-bold text-[var(--color-text-main)] mt-1 uppercase">{modelData.thinking || 'off'}</p>
          </div>
        </div>
      {/if}

      {#if modelMessage}
        <div class="p-2.5 rounded-xl bg-blue-500/10 text-xs font-semibold text-[var(--color-text-main)]">
          {modelMessage}
        </div>
      {/if}
    </div>

    <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">Installed Agent Skills</h3>
        <span class="text-xs font-semibold text-[var(--color-primary)]">{skillsList.length} skills loaded</span>
      </div>

      {#if skillsLoading}
        <p class="text-xs sm:text-sm text-[var(--color-text-muted)]">Loading skills directory...</p>
      {:else if skillsList.length === 0}
        <p class="text-xs text-[var(--color-text-muted)]">No custom skills in ~/.ryvos/skills.</p>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-2.5 sm:gap-4">
          {#each skillsList as skill}
            <div class="neu-inset p-2.5 sm:p-4 rounded-xl space-y-1">
              <span class="font-bold text-xs text-[var(--color-text-main)]">{skill.name}</span>
              <p class="text-xs text-[var(--color-text-muted)]">{skill.description || 'Custom capability'}</p>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>