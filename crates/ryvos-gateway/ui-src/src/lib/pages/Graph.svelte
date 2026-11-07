<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let jobs = $state([]);
  let loading = $state(true);
  let error = $state('');
  let showAddForm = $state(false);
  let newJob = $state({ name: '', schedule: '', prompt: '', channel: '', goal: '' });
  let saving = $state(false);
  let message = $state('');

  onMount(async () => {
    await loadJobs();
  });

  async function loadJobs() {
    loading = true;
    try {
      const data = await apiFetch('/api/cron');
      jobs = data.jobs || [];
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
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
        message = res.note || 'Job added';
        newJob = { name: '', schedule: '', prompt: '', channel: '', goal: '' };
        showAddForm = false;
        await loadJobs();
      } else {
        message = res.error || 'Failed to add job';
      }
    } catch (e) {
      message = 'Error: ' + e.message;
    } finally {
      saving = false;
    }
  }

  async function deleteJob(name) {
    try {
      const res = await apiFetch(`/api/cron/${encodeURIComponent(name)}`, { method: 'DELETE' });
      message = res.note || 'Job removed';
      await loadJobs();
    } catch (e) {
      message = 'Error: ' + e.message;
    }
  }

  function truncate(s, max) {
    if (!s) return '';
    return s.length > max ? s.substring(0, max) + '...' : s;
  }
</script>

<div>
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Cron Jobs</h2>
      <p class="text-[#778399] text-sm mt-1">Scheduled autonomous agent execution tasks</p>
    </div>
    <button
      type="button"
      onclick={() => showAddForm = !showAddForm}
      class="btn-primary px-4 py-2 text-xs font-semibold"
    >
      {showAddForm ? 'Cancel' : 'Add Job'}
    </button>
  </div>

  {#if showAddForm}
    <div class="card-soft p-6 mb-5">
      <h3 class="text-sm font-heading font-bold text-[#1B2943] mb-4">New Cron Job</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="cron-name" class="label block mb-1">Job Name</label>
          <input
            id="cron-name"
            bind:value={newJob.name}
            placeholder="daily-summary"
            class="input-soft w-full px-3 py-2 text-sm"
          />
        </div>
        <div>
          <label for="cron-schedule" class="label block mb-1">Schedule (Cron Syntax)</label>
          <input
            id="cron-schedule"
            bind:value={newJob.schedule}
            placeholder="0 8 * * *"
            class="input-soft w-full px-3 py-2 text-sm font-mono"
          />
        </div>
        <div class="md:col-span-2">
          <label for="cron-prompt" class="label block mb-1">Task Prompt</label>
          <textarea
            id="cron-prompt"
            bind:value={newJob.prompt}
            placeholder="Summarize my unread emails and create a daily briefing..."
            rows="3"
            class="input-soft w-full px-3 py-2 text-sm resize-none"
          ></textarea>
        </div>
        <div>
          <label for="cron-channel" class="label block mb-1">Channel (optional)</label>
          <input
            id="cron-channel"
            bind:value={newJob.channel}
            placeholder="telegram"
            class="input-soft w-full px-3 py-2 text-sm"
          />
        </div>
        <div>
          <label for="cron-goal" class="label block mb-1">Goal (optional)</label>
          <input
            id="cron-goal"
            bind:value={newJob.goal}
            placeholder="Keep me informed"
            class="input-soft w-full px-3 py-2 text-sm"
          />
        </div>
      </div>
      <div class="flex items-center gap-3 mt-5">
        <button
          type="button"
          onclick={addJob}
          disabled={saving || !newJob.name || !newJob.schedule || !newJob.prompt}
          class="btn-primary px-5 py-2 text-xs font-semibold disabled:opacity-40"
        >
          {saving ? 'Saving...' : 'Save Job'}
        </button>
        <button
          type="button"
          onclick={() => showAddForm = false}
          class="btn-surface px-4 py-2 text-xs font-semibold text-[#778399]"
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading cron jobs...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">{error}</p>
    </div>
  {:else if jobs.length === 0}
    <div class="card-soft p-12 text-center">
      <div class="w-12 h-12 rounded-2xl bg-[#E8EEFF] text-[#2448CB] flex items-center justify-center mx-auto mb-3">
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
      </div>
      <h3 class="text-base font-semibold text-[#1B2943] mb-1">No cron jobs configured</h3>
      <p class="text-xs text-[#778399] max-w-sm mx-auto">
        Add a cron job to schedule recurring agent tasks on a timed schedule.
      </p>
    </div>
  {:else}
    <div class="card-soft overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
              {#each ['Name', 'Schedule', 'Prompt', 'Channel', 'Goal', 'Actions'] as col}
                <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">
                  {col}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody class="divide-y divide-[#DCE4F3]">
            {#each jobs as job}
              <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                <td class="px-4 py-3 font-semibold text-[#1B2943]">{job.name || '-'}</td>
                <td class="px-4 py-3 font-mono text-xs text-[#64748B] tabular-nums">{job.schedule || '-'}</td>
                <td class="px-4 py-3 text-[#64748B] max-w-[240px] truncate" title={job.prompt || ''}>{truncate(job.prompt || '', 50)}</td>
                <td class="px-4 py-3 text-[#778399]">{job.channel || '-'}</td>
                <td class="px-4 py-3 text-[#778399]">{job.goal || '-'}</td>
                <td class="px-4 py-3">
                  <button
                    type="button"
                    onclick={() => deleteJob(job.name)}
                    class="text-xs font-semibold text-[#D94B5B] hover:bg-[#FEECEE] px-2.5 py-1 rounded-lg transition-colors"
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

  {#if message}
    <div class="mt-4 px-4 py-3 bg-[#E8EEFF] border border-[#2448CB]/30 rounded-xl text-sm text-[#1B2943] flex items-center justify-between">
      <span>{message}</span>
      <button type="button" onclick={() => message = ''} class="text-[#778399] hover:text-[#1B2943] text-xs font-semibold uppercase">Dismiss</button>
    </div>
  {/if}

  {#if message && (message.includes('added') || message.includes('removed'))}
    <div class="mt-2.5 px-4 py-2 bg-[#FFFBEB] border border-[#C58A2A]/30 rounded-xl text-xs text-[#C58A2A] font-medium">
      Restart required for cron changes to take effect.
    </div>
  {/if}
</div>
