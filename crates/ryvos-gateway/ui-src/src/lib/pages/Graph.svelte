<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let activeTab = $state('cron');
  let jobs = $state([]);
  let heartbeats = $state([]);
  let skills = $state([]);
  let loading = $state(true);
  let error = $state('');
  let showAddForm = $state(false);
  let newJob = $state({ name: '', schedule: '', prompt: '', channel: '', goal: '' });
  let saving = $state(false);
  let message = $state('');

  async function loadData() {
    loading = true;
    error = '';
    try {
      if (activeTab === 'cron') {
        const data = await apiFetch('/api/cron');
        jobs = data.jobs || [];
      } else if (activeTab === 'heartbeat') {
        const data = await apiFetch('/api/heartbeat/history');
        heartbeats = data.events || [];
      } else if (activeTab === 'skills') {
        const data = await apiFetch('/api/skills');
        skills = data.skills || [];
      }
    } catch (e) {
      error = e.message || 'Failed to load task records';
    } finally {
      loading = false;
    }
  }

  function setTab(tab) {
    activeTab = tab;
    loadData();
  }

  async function addJob() {
    if (!newJob.name || !newJob.schedule || !newJob.prompt) return;
    saving = true;
    message = '';
    try {
      const body = { ...newJob };
      if (!body.channel) delete body.channel;
      if (!body.goal) delete body.goal;
      const res = await apiFetch('/api/cron', { method: 'POST', body: JSON.stringify(body) });
      if (res.ok) {
        message = res.note || 'Scheduled job registered successfully';
        newJob = { name: '', schedule: '', prompt: '', channel: '', goal: '' };
        showAddForm = false;
        await loadData();
      } else {
        message = res.error || 'Failed to register scheduled job';
      }
    } catch (e) {
      message = 'Error: ' + e.message;
    } finally {
      saving = false;
    }
  }

  async function deleteJob(name) {
    try {
      const res = await apiFetch('/api/cron/' + encodeURIComponent(name), { method: 'DELETE' });
      message = res.note || 'Scheduled job removed';
      await loadData();
    } catch (e) {
      message = 'Error: ' + e.message;
    }
  }

  function formatTime(isoStr) {
    if (!isoStr) return 'N/A';
    try {
      return new Date(isoStr).toLocaleString([], {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return String(isoStr);
    }
  }

  function truncate(s, max) {
    if (!s) return 'None';
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="space-y-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div>
      <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Execution Schedules and Skills</h2>
      <p class="text-sm text-[var(--color-text-muted)] mt-1">Autonomous recurring cron routines, heartbeat watchdog events, and loaded agent skills</p>
    </div>
    {#if activeTab === 'cron'}
      <button
        type="button"
        onclick={() => { showAddForm = !showAddForm; }}
        class="btn-primary min-h-[44px] px-5 py-2 text-xs font-semibold rounded-xl"
      >
        {showAddForm ? 'Close Form' : 'Add Scheduled Routine'}
      </button>
    {/if}
  </div>

  <div class="neu-card p-2 flex items-center gap-2 overflow-x-auto" role="tablist" aria-label="Task views">
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'cron'}
      onclick={() => setTab('cron')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'cron' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Cron Schedules
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'heartbeat'}
      onclick={() => setTab('heartbeat')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'heartbeat' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Heartbeat Watchdog
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'skills'}
      onclick={() => setTab('skills')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'skills' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Loaded Skills ({skills.length})
    </button>
  </div>

  {#if showAddForm && activeTab === 'cron'}
    <div class="neu-card p-6 space-y-4">
      <h3 class="text-sm font-bold uppercase tracking-wider text-[var(--color-text-main)]">Register New Schedule</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="space-y-1">
          <label for="cron-name" class="text-xs font-semibold text-[var(--color-text-muted)]">Routine Name</label>
          <input
            id="cron-name"
            bind:value={newJob.name}
            placeholder="system-health-check"
            class="neu-inset min-h-[44px] w-full px-3 py-2 text-sm rounded-xl text-[var(--color-text-main)] font-mono"
          />
        </div>
        <div class="space-y-1">
          <label for="cron-schedule" class="text-xs font-semibold text-[var(--color-text-muted)]">Cron Expression</label>
          <input
            id="cron-schedule"
            bind:value={newJob.schedule}
            placeholder="0 8 * * *"
            class="neu-inset min-h-[44px] w-full px-3 py-2 text-sm rounded-xl text-[var(--color-text-main)] font-mono"
          />
        </div>
        <div class="md:col-span-2 space-y-1">
          <label for="cron-prompt" class="text-xs font-semibold text-[var(--color-text-muted)]">Instruction Prompt</label>
          <textarea
            id="cron-prompt"
            bind:value={newJob.prompt}
            placeholder="Describe what the agent runtime should execute on each tick..."
            rows="3"
            class="neu-inset w-full p-3 text-sm rounded-xl text-[var(--color-text-main)] font-mono resize-none min-h-[88px]"
          ></textarea>
        </div>
        <div class="space-y-1">
          <label for="cron-channel" class="text-xs font-semibold text-[var(--color-text-muted)]">Broadcast Channel (Optional)</label>
          <input
            id="cron-channel"
            bind:value={newJob.channel}
            placeholder="telegram"
            class="neu-inset min-h-[44px] w-full px-3 py-2 text-sm rounded-xl text-[var(--color-text-main)]"
          />
        </div>
        <div class="space-y-1">
          <label for="cron-goal" class="text-xs font-semibold text-[var(--color-text-muted)]">Linked Objective (Optional)</label>
          <input
            id="cron-goal"
            bind:value={newJob.goal}
            placeholder="Infrastructure Monitoring"
            class="neu-inset min-h-[44px] w-full px-3 py-2 text-sm rounded-xl text-[var(--color-text-main)]"
          />
        </div>
      </div>
      <div class="flex items-center gap-3 pt-2">
        <button
          type="button"
          onclick={addJob}
          disabled={saving || !newJob.name || !newJob.schedule || !newJob.prompt}
          class="btn-primary min-h-[44px] px-6 py-2 text-xs font-semibold rounded-xl disabled:opacity-40"
        >
          {saving ? 'Registering...' : 'Save Schedule'}
        </button>
        <button
          type="button"
          onclick={() => { showAddForm = false; }}
          class="btn-soft min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl text-[var(--color-text-muted)]"
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if message}
    <div class="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20 text-sm text-[var(--color-text-main)] flex items-center justify-between">
      <span>{message}</span>
      <button type="button" onclick={() => { message = ''; }} class="text-xs font-semibold uppercase text-[var(--color-text-muted)]">Dismiss</button>
    </div>
  {/if}

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading task data...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Schedule Error</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else if activeTab === 'cron'}
    {#if jobs.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Scheduled Routines</p>
        <p class="text-xs text-[var(--color-text-muted)]">Register a recurring cron schedule above to run automated tasks.</p>
      </div>
    {:else}
      <div class="neu-card p-0 overflow-hidden">
        <div class="table-scroll">
          <table class="w-full text-left text-sm">
            <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
              <tr>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Routine Name</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Schedule</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Instruction Prompt</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Target Channel</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
              {#each jobs as job}
                <tr class="hover:bg-[var(--color-bg)] transition-colors">
                  <td class="px-4 py-3 font-semibold text-xs text-[var(--color-text-main)] whitespace-nowrap">
                    {job.name || 'Unnamed'}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs text-[var(--color-primary)] tabular-nums whitespace-nowrap">
                    {job.schedule || 'N/A'}
                  </td>
                  <td class="px-4 py-3 text-xs text-[var(--color-text-muted)] max-w-sm truncate" title={job.prompt}>
                    {truncate(job.prompt, 55)}
                  </td>
                  <td class="px-4 py-3 text-xs text-[var(--color-text-muted)] whitespace-nowrap">
                    {job.channel || 'Internal'}
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap">
                    <button
                      type="button"
                      onclick={() => deleteJob(job.name)}
                      class="btn-danger min-h-[36px] px-3 py-1 text-xs font-semibold rounded-lg"
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

  {:else if activeTab === 'heartbeat'}
    {#if heartbeats.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Heartbeat Telemetry</p>
        <p class="text-xs text-[var(--color-text-muted)]">Heartbeat watchdog cycles will be recorded here when active.</p>
      </div>
    {:else}
      <div class="neu-card p-0 overflow-hidden">
        <div class="table-scroll">
          <table class="w-full text-left text-sm">
            <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
              <tr>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Timestamp</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Session</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Tool Called</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Outcome</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
              {#each heartbeats as hb}
                <tr class="hover:bg-[var(--color-bg)] transition-colors">
                  <td class="px-4 py-3 font-mono text-xs text-[var(--color-text-muted)] tabular-nums whitespace-nowrap">
                    {formatTime(hb.timestamp || hb.time)}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs text-[var(--color-text-main)] whitespace-nowrap">
                    {truncate(hb.session_id, 24)}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs font-bold text-[var(--color-primary)] whitespace-nowrap">
                    {hb.tool || hb.tool_name || 'watchdog'}
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap">
                    <span class="inline-flex px-2 py-0.5 rounded-full text-xs font-semibold badge-success">
                      Verified
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

  {:else if activeTab === 'skills'}
    {#if skills.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Installed Agent Skills</p>
        <p class="text-xs text-[var(--color-text-muted)]">Skills located in ~/.ryvos/skills will appear here automatically.</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each skills as skill}
          <div class="neu-card p-5 space-y-2">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-bold text-[var(--color-text-main)]">{skill.name}</h3>
              <span class="inline-flex px-2 py-0.5 rounded-full text-xs font-semibold badge-info">
                Active
              </span>
            </div>
            <p class="text-xs text-[var(--color-text-muted)] leading-relaxed">{skill.description || 'Custom autonomous agent capability skill'}</p>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>