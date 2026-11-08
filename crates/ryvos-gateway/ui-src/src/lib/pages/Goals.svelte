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
  let judgeEvents = $state([]);
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
    judgeEvents = [];

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
      error = e.message || 'Failed to start goal run';
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
      } else if (latest.kind === 'judge_verdict' || latest.kind === 'goal_evaluated') {
        judgeEvents = [{ title: latest.text, data: latest.data, time: latest.time }, ...judgeEvents].slice(0, 10);
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

<div class="space-y-4 sm:space-y-6">
  <div>
    <h2 class="text-sm sm:text-base font-bold tracking-tight text-[var(--color-text-main)]">Autonomous Goals Orchestration</h2>
    <p class="text-xs text-[var(--color-text-muted)] mt-0.5">Autonomous OODA execution loops with runtime self-evaluation and telemetry</p>
  </div>

  <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4">
    <div>
      <label for="goal-description" class="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-1.5">
        Goal Objective
      </label>
      <textarea
        id="goal-description"
        bind:value={goalDescription}
        placeholder="Specify the objective for the autonomous agent runtime to plan, execute, and verify..."
        rows="3"
        class="neu-inset w-full p-2.5 sm:p-3 text-xs sm:text-sm font-mono rounded-xl resize-none text-[var(--color-text-main)] min-h-[80px]"
      ></textarea>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-1">
      <div class="flex items-center gap-2">
        <label for="goal-channel" class="text-xs font-semibold text-[var(--color-text-muted)] whitespace-nowrap">
          Notification Route:
        </label>
        <select
          id="goal-channel"
          bind:value={channel}
          class="neu-inset min-h-[36px] px-2.5 py-1.5 text-xs sm:text-sm rounded-lg text-[var(--color-text-main)] bg-[var(--color-surface)]"
        >
          <option value="">Gateway Control Plane Only</option>
          <option value="telegram">Telegram Broadcast</option>
        </select>
      </div>

      <button
        type="button"
        onclick={executeGoal}
        disabled={executing || !goalDescription.trim()}
        class="btn-primary min-h-[36px] px-4 py-1.5 rounded-lg text-xs sm:text-sm font-semibold disabled:opacity-40"
      >
        {executing ? 'Executing Objective...' : 'Launch Goal Loop'}
      </button>
    </div>
  </div>

  {#if executing || result || error}
    <div class="neu-card p-3 sm:p-5 space-y-3 sm:space-y-4 border-l-4 {error ? 'border-[var(--color-danger)]' : executing ? 'border-[var(--color-primary)]' : 'border-[var(--color-success)]'}">
      <div class="flex items-center justify-between">
        <h3 class="text-xs sm:text-sm font-bold uppercase tracking-wider text-[var(--color-text-main)]">Live Execution Status</h3>
        {#if executing}
          <span class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-info animate-pulse">
            Active Run
          </span>
        {:else if error}
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-danger">
            Failed
          </span>
        {:else}
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-success">
            Success
          </span>
        {/if}
      </div>

      {#if activeSession}
        <div class="text-xs font-mono text-[var(--color-text-muted)]">
          Session ID: <span class="text-[var(--color-text-main)]">{activeSession}</span>
        </div>
      {/if}

      {#if judgeEvents.length > 0}
        <div class="neu-inset p-2.5 sm:p-3 rounded-xl space-y-1.5">
          <div class="text-xs font-bold text-[var(--color-primary)]">Judge and Evaluator Verdicts</div>
          {#each judgeEvents as je}
            <div class="text-xs flex items-center justify-between text-[var(--color-text-main)]">
              <span>{je.title}</span>
              <span class="font-mono tabular-nums text-[var(--color-text-muted)]">{je.time}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if toolEvents.length > 0}
        <div class="neu-inset p-2.5 sm:p-3 rounded-xl max-h-48 overflow-y-auto space-y-1 font-mono text-xs">
          {#each toolEvents as evt}
            <div class="flex items-center gap-2 py-0.5">
              <span class={evt.kind === 'start' ? 'text-[var(--color-primary)] font-bold' : 'text-[var(--color-success)] font-bold'}>
                {evt.kind === 'start' ? '>>>' : '<<<'}
              </span>
              <span class="text-[var(--color-text-main)]">{evt.tool}</span>
              <span class="ml-auto text-[10px] text-[var(--color-text-muted)] tabular-nums">{evt.time}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if currentStream && executing}
        <div class="space-y-1">
          <div class="text-xs font-semibold text-[var(--color-text-muted)]">Live Stream Output:</div>
          <pre class="p-3 rounded-xl neu-inset text-xs font-mono whitespace-pre-wrap leading-relaxed text-[var(--color-text-main)] max-h-64 overflow-y-auto">{currentStream}</pre>
        </div>
      {/if}

      {#if result && !executing}
        <div class="space-y-1">
          <div class="text-xs font-semibold text-[var(--color-success)]">Execution Outcome:</div>
          <pre class="p-3 rounded-xl neu-inset text-xs font-mono whitespace-pre-wrap leading-relaxed text-[var(--color-text-main)] max-h-64 overflow-y-auto">{result}</pre>
        </div>
      {/if}

      {#if error}
        <div class="p-2.5 rounded-xl bg-red-500/10 text-xs font-mono text-[var(--color-danger)]">
          {error}
        </div>
      {/if}
    </div>
  {/if}

  <div class="space-y-3">
    <h3 class="text-sm sm:text-base font-bold text-[var(--color-text-main)]">Goal Execution Archives</h3>
    {#if historyLoading}
      <div class="neu-card p-4 sm:p-6 text-center text-xs sm:text-sm text-[var(--color-text-muted)]">
        Loading historical runs...
      </div>
    {:else if history.length === 0}
      <div class="neu-card p-4 sm:p-6 text-center text-xs sm:text-sm text-[var(--color-text-muted)]">
        No previous runs archived. Execute a goal above to start telemetry collection.
      </div>
    {:else}
      <div class="neu-card p-0 overflow-hidden">
        <div class="table-scroll overflow-x-auto">
          <table class="w-full text-left text-xs">
            <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
              <tr>
                <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Timestamp</th>
                <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Session</th>
                <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Turns</th>
                <th class="px-2.5 py-2 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Status</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
              {#each history as run}
                <tr class="hover:bg-[var(--color-bg)] transition-colors">
                  <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-muted)] whitespace-nowrap">
                    {formatTime(run.start_time)}
                  </td>
                  <td class="px-2.5 py-2 font-mono text-xs text-[var(--color-text-main)]">
                    {(run.session_id || '').substring(0, 24)}
                  </td>
                  <td class="px-2.5 py-2 font-mono text-xs tabular-nums text-[var(--color-text-main)]">
                    {run.total_turns || 0}
                  </td>
                  <td class="px-2.5 py-2 whitespace-nowrap">
                    {#if run.status === 'complete' || run.status === 'success'}
                      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-success">
                        Completed
                      </span>
                    {:else if run.status === 'running'}
                      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-warning">
                        Running
                      </span>
                    {:else}
                      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] sm:text-xs font-semibold badge-danger">
                        {run.status || 'Failed'}
                      </span>
                    {/if}
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