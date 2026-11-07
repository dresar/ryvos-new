<script>
  import { onMount, onDestroy } from 'svelte';
  import { apiFetch } from '../api.js';
  import { activityFeed, streamingText } from '../ws.js';

  let goalDescription = $state('');
  let channel = $state('');
  let executing = $state(false);
  let activeSession = $state('');
  let result = $state('');
  let error = $state('');
  let currentStream = $state('');
  let toolEvents = $state([]);
  let history = $state([]);
  let historyLoading = $state(true);

  let unsubStream = () => {};
  let unsubActivity = () => {};

  async function executeGoal() {
    if (!goalDescription.trim()) return;
    executing = true;
    result = '';
    error = '';
    currentStream = '';
    toolEvents = [];

    try {
      const data = await apiFetch('/api/goals/run', {
        method: 'POST',
        body: JSON.stringify({
          description: goalDescription,
          channel: channel || undefined,
        }),
      });
      activeSession = data.session_id;
    } catch (e) {
      error = e.message;
      executing = false;
    }
  }

  async function loadHistory() {
    historyLoading = true;
    try {
      const data = await apiFetch('/api/goals/history');
      history = data.runs || [];
    } catch (e) {
      history = [];
    } finally {
      historyLoading = false;
    }
  }

  function formatTime(isoStr) {
    if (!isoStr) return '-';
    return new Date(isoStr).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }

  onMount(() => {
    unsubStream = streamingText.subscribe(v => {
      if (executing && v) currentStream = v;
    });

    unsubActivity = activityFeed.subscribe(feed => {
      if (!executing || feed.length === 0) return;
      const latest = feed[0];
      if (latest.kind === 'tool_start') {
        toolEvents = [{ kind: 'start', tool: latest.text, time: latest.time }, ...toolEvents].slice(0, 30);
      } else if (latest.kind === 'tool_end') {
        toolEvents = [{ kind: 'end', tool: latest.text, time: latest.time }, ...toolEvents].slice(0, 30);
      } else if (latest.kind === 'run_complete') {
        executing = false;
        result = currentStream || 'Goal execution completed.';
        loadHistory();
      } else if (latest.kind === 'run_error') {
        executing = false;
        error = latest.text || 'Goal execution failed.';
      }
    });

    loadHistory();
  });

  onDestroy(() => {
    unsubStream();
    unsubActivity();
  });
</script>

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Goals</h2>
    <p class="text-[#778399] text-sm mt-1">Director-driven goal orchestration: OODA loop with auto-evolving execution graphs.</p>
  </div>

  <div class="card-soft p-6 mb-6">
    <label for="goal-description" class="label block mb-2 text-[#1B2943]">Goal Description</label>
    <textarea
      id="goal-description"
      bind:value={goalDescription}
      placeholder="Describe the objective for the autonomous agent to plan and execute..."
      rows="3"
      class="input-soft w-full px-4 py-3 text-sm font-mono resize-none"
    ></textarea>

    <div class="flex flex-col sm:flex-row sm:items-center gap-4 mt-4">
      <div class="flex-1">
        <label for="goal-channel" class="label block mb-1 text-[10px]">Route Result Notification</label>
        <select
          id="goal-channel"
          bind:value={channel}
          class="input-soft px-3 py-2 text-sm w-full sm:w-auto min-w-[200px]"
        >
          <option value="">None (Gateway UI only)</option>
          <option value="telegram">Telegram</option>
        </select>
      </div>
      <button
        type="button"
        onclick={executeGoal}
        disabled={executing || !goalDescription.trim()}
        class="btn-primary px-8 py-3 text-sm font-semibold disabled:opacity-40"
      >
        {executing ? 'Executing...' : 'Execute Goal'}
      </button>
    </div>
  </div>

  {#if executing || result || error}
    <div class="card-soft p-6 mb-6">
      <div class="flex items-center justify-between mb-3">
        <span class="label text-[#1B2943]">Execution Progress</span>
        {#if executing}
          <span class="inline-flex items-center gap-1.5 text-xs font-semibold text-[#2448CB]">
            <span class="w-2 h-2 bg-[#2448CB] rounded-full animate-pulse"></span>
            Running
          </span>
        {:else if error}
          <span class="text-xs font-semibold text-[#D94B5B]">Failed</span>
        {:else}
          <span class="text-xs font-semibold text-[#219B70]">Complete</span>
        {/if}
      </div>

      {#if activeSession}
        <p class="text-xs font-mono text-[#778399] mb-3">Session ID: {activeSession}</p>
      {/if}

      {#if toolEvents.length > 0}
        <div class="mb-4 max-h-40 overflow-y-auto space-y-1 p-2 bg-[#F8FAFF] rounded-xl border border-[#DCE4F3]">
          {#each toolEvents as evt}
            <div class="flex items-center gap-2 py-0.5 text-xs font-mono">
              <span class={evt.kind === 'start' ? 'text-[#2448CB] font-bold' : 'text-[#219B70] font-bold'}>
                {evt.kind === 'start' ? '>' : '<'}
              </span>
              <span class="text-[#1B2943]">{evt.tool}</span>
              <span class="ml-auto text-[10px] text-[#778399] tabular-nums">{evt.time}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if currentStream && executing}
        <div class="border-t border-[#DCE4F3] pt-3 mt-3">
          <pre class="text-sm text-[#1B2943] bg-[#F8FAFF] border border-[#DCE4F3] whitespace-pre-wrap font-mono leading-relaxed">{currentStream}</pre>
        </div>
      {/if}

      {#if result && !executing}
        <div class="border-t border-[#DCE4F3] pt-3 mt-3">
          <pre class="text-sm text-[#1B2943] bg-[#F8FAFF] border border-[#DCE4F3] whitespace-pre-wrap font-mono leading-relaxed">{result}</pre>
        </div>
      {/if}

      {#if error}
        <div class="border-t border-[#D94B5B]/30 pt-3 mt-3">
          <p class="text-sm text-[#D94B5B] font-mono">{error}</p>
        </div>
      {/if}
    </div>
  {/if}

  <div>
    <h3 class="text-lg font-heading font-bold text-[#1B2943] mb-3">Goal Run History</h3>
    {#if historyLoading}
      <div class="card-soft p-8 text-center">
        <p class="text-[#778399] text-sm animate-pulse">Loading history...</p>
      </div>
    {:else if history.length === 0}
      <div class="card-soft p-8 text-center">
        <p class="text-[#778399] text-sm">No goal runs recorded yet. Execute your first goal above.</p>
      </div>
    {:else}
      <div class="card-soft overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-[#F8FAFF] border-b border-[#DCE4F3]">
                {#each ['Time', 'Session', 'Turns', 'Status'] as col}
                  <th class="px-4 py-3 text-left text-xs uppercase tracking-wider font-semibold text-[#778399]">{col}</th>
                {/each}
              </tr>
            </thead>
            <tbody class="divide-y divide-[#DCE4F3]">
              {#each history as run}
                <tr class="hover:bg-[#F8FAFF] transition-colors duration-150">
                  <td class="px-4 py-3 font-mono text-xs text-[#778399] tabular-nums">{formatTime(run.start_time)}</td>
                  <td class="px-4 py-3 font-mono text-xs text-[#778399]">{(run.session_id || '').substring(0, 20)}</td>
                  <td class="px-4 py-3 text-[#1B2943] tabular-nums">{run.total_turns || 0}</td>
                  <td class="px-4 py-3">
                    <span class="font-semibold text-xs {run.status === 'complete' ? 'text-[#219B70]' : run.status === 'running' ? 'text-[#C58A2A]' : 'text-[#D94B5B]'}">
                      {run.status || '-'}
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  </div>
</div>
