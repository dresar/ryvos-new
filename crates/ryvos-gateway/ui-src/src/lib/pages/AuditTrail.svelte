<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let activeTab = $state('audit');
  let entries = $state([]);
  let lessons = $state([]);
  let decisions = $state([]);
  let failures = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filterTool = $state('');
  let filterSearch = $state('');
  let expandedRow = $state(null);

  function formatOutcome(outcome) {
    if (!outcome) return { label: 'Unknown', badgeClass: 'badge-neutral', detail: '' };
    if (typeof outcome === 'string') {
      const lower = outcome.toLowerCase();
      if (lower === 'harmless' || lower === 'ok' || lower === 'success') {
        return { label: 'Harmless', badgeClass: 'badge-success', detail: '' };
      }
      if (lower === 'error' || lower === 'failed') {
        return { label: 'Error', badgeClass: 'badge-danger', detail: '' };
      }
      return { label: outcome, badgeClass: 'badge-info', detail: '' };
    }
    if (typeof outcome === 'object') {
      if (outcome.Harmless !== undefined) {
        return { label: 'Harmless', badgeClass: 'badge-success', detail: '' };
      }
      if (outcome.NearMiss) {
        return {
          label: 'Near Miss',
          badgeClass: 'badge-warning',
          detail: outcome.NearMiss.what_could_have_happened || 'Potential hazard avoided'
        };
      }
      if (outcome.Incident) {
        const sev = outcome.Incident.severity ? ' (' + outcome.Incident.severity + ')' : '';
        return {
          label: 'Incident' + sev,
          badgeClass: 'badge-danger',
          detail: outcome.Incident.what_happened || 'Safety violation recorded'
        };
      }
      if (outcome.UserCorrected) {
        return {
          label: 'User Corrected',
          badgeClass: 'badge-warning',
          detail: outcome.UserCorrected.feedback || 'User intervention applied'
        };
      }
      return { label: 'Object', badgeClass: 'badge-info', detail: JSON.stringify(outcome) };
    }
    return { label: String(outcome), badgeClass: 'badge-info', detail: '' };
  }

  function formatTime(isoStr) {
    if (!isoStr) return 'N/A';
    try {
      return new Date(isoStr).toLocaleString([], {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      });
    } catch {
      return String(isoStr);
    }
  }

  function truncate(s, max) {
    if (!s) return 'None';
    if (typeof s === 'object') s = JSON.stringify(s);
    return s.length > max ? s.substring(0, max) + '...' : s;
  }

  async function loadData() {
    loading = true;
    error = '';
    try {
      if (activeTab === 'audit') {
        let url = '/api/audit?limit=50';
        if (filterTool) url += '&tool=' + encodeURIComponent(filterTool);
        const res = await apiFetch(url);
        entries = res.entries || res.events || res.audit || [];
      } else if (activeTab === 'lessons') {
        let url = '/api/safety/lessons?limit=50';
        if (filterSearch) url += '&search=' + encodeURIComponent(filterSearch);
        const res = await apiFetch(url);
        lessons = res.lessons || [];
      } else if (activeTab === 'decisions') {
        const res = await apiFetch('/api/decisions?limit=50');
        decisions = res.decisions || [];
      } else if (activeTab === 'failures') {
        let url = '/api/failures?limit=50';
        if (filterSearch) url += '&pattern=' + encodeURIComponent(filterSearch);
        const res = await apiFetch(url);
        failures = res.failures || [];
      }
    } catch (e) {
      error = e.message || 'Failed to fetch audit data';
    } finally {
      loading = false;
    }
  }

  function setTab(tab) {
    activeTab = tab;
    expandedRow = null;
    loadData();
  }

  function toggleRow(idx) {
    expandedRow = expandedRow === idx ? null : idx;
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Audit Trail and Safety Journal</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Immutable security log of tool executions, safety policies, decisions, and failure records</p>
  </div>

  <div class="neu-card p-2 flex items-center gap-2 overflow-x-auto" role="tablist" aria-label="Audit views">
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'audit'}
      onclick={() => setTab('audit')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'audit' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Tool Executions
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'lessons'}
      onclick={() => setTab('lessons')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'lessons' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Safety Lessons
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'decisions'}
      onclick={() => setTab('decisions')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'decisions' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Decisions Journal
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'failures'}
      onclick={() => setTab('failures')}
      class="min-h-[44px] px-4 py-2 rounded-xl text-sm font-semibold transition-all {activeTab === 'failures' ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
    >
      Failure Journal
    </button>
  </div>

  {#if activeTab === 'audit'}
    <div class="neu-card p-4 flex flex-wrap items-center gap-3">
      <label for="filter-tool-input" class="text-xs font-semibold text-[var(--color-text-muted)]">
        Filter Tool:
      </label>
      <input
        id="filter-tool-input"
        type="text"
        bind:value={filterTool}
        placeholder="e.g. bash, web_fetch, write_file"
        onkeydown={(e) => { if (e.key === 'Enter') loadData(); }}
        class="neu-inset min-h-[44px] px-4 py-2 text-sm text-[var(--color-text-main)] rounded-xl w-64 font-mono"
      />
      <button
        type="button"
        onclick={loadData}
        class="btn-primary min-h-[44px] px-4 py-2 rounded-xl text-xs font-semibold"
      >
        Filter
      </button>
      {#if filterTool}
        <button
          type="button"
          onclick={() => { filterTool = ''; loadData(); }}
          class="btn-soft min-h-[44px] px-4 py-2 rounded-xl text-xs font-semibold"
        >
          Reset
        </button>
      {/if}
    </div>
  {:else if activeTab === 'lessons' || activeTab === 'failures'}
    <div class="neu-card p-4 flex flex-wrap items-center gap-3">
      <label for="filter-search-input" class="text-xs font-semibold text-[var(--color-text-muted)]">
        Search Query:
      </label>
      <input
        id="filter-search-input"
        type="text"
        bind:value={filterSearch}
        placeholder="Search keywords or patterns..."
        onkeydown={(e) => { if (e.key === 'Enter') loadData(); }}
        class="neu-inset min-h-[44px] px-4 py-2 text-sm text-[var(--color-text-main)] rounded-xl w-72"
      />
      <button
        type="button"
        onclick={loadData}
        class="btn-primary min-h-[44px] px-4 py-2 rounded-xl text-xs font-semibold"
      >
        Search
      </button>
      {#if filterSearch}
        <button
          type="button"
          onclick={() => { filterSearch = ''; loadData(); }}
          class="btn-soft min-h-[44px] px-4 py-2 rounded-xl text-xs font-semibold"
        >
          Reset
        </button>
      {/if}
    </div>
  {/if}

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading security log records...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Audit Fetch Error</h3>
      <p class="text-sm text-[var(--color-text-muted)] font-mono">{error}</p>
      <button
        type="button"
        onclick={loadData}
        class="btn-soft min-h-[44px] px-4 py-2 mt-4 rounded-xl text-xs font-semibold"
      >
        Retry Request
      </button>
    </div>
  {:else if activeTab === 'audit'}
    {#if entries.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Tool Execution Logs</p>
        <p class="text-xs text-[var(--color-text-muted)]">Tool actions executed by the agent runtime will be recorded here.</p>
      </div>
    {:else}
      <div class="neu-card p-0 overflow-hidden">
        <div class="table-scroll">
          <table class="w-full text-left text-sm">
            <thead class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
              <tr>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Timestamp</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Tool</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Parameters</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Outcome</th>
                <th class="px-4 py-3 text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider">Details</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-[var(--color-border)]">
              {#each entries as entry, idx}
                {@const out = formatOutcome(entry.outcome || (entry.error ? 'error' : 'ok'))}
                <tr class="hover:bg-[var(--color-bg)] transition-colors cursor-pointer" onclick={() => toggleRow(idx)}>
                  <td class="px-4 py-3 font-mono text-xs tabular-nums text-[var(--color-text-muted)] whitespace-nowrap">
                    {formatTime(entry.timestamp || entry.time)}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs font-bold text-[var(--color-primary)] whitespace-nowrap">
                    {entry.tool || entry.tool_name || 'N/A'}
                  </td>
                  <td class="px-4 py-3 font-mono text-xs text-[var(--color-text-muted)] max-w-xs truncate" title={typeof entry.input === 'object' ? JSON.stringify(entry.input) : entry.input}>
                    {truncate(entry.input || entry.params, 50)}
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap">
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold {out.badgeClass}">
                      {out.label}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-xs text-[var(--color-primary)] font-semibold whitespace-nowrap">
                    {expandedRow === idx ? 'Collapse' : 'Expand'}
                  </td>
                </tr>
                {#if expandedRow === idx}
                  <tr class="bg-[var(--color-bg)]">
                    <td colspan="5" class="p-4 space-y-3">
                      <div class="neu-card p-4 space-y-2 text-xs">
                        {#if out.detail}
                          <div class="text-[var(--color-danger)] font-semibold mb-2">
                            Reason: {out.detail}
                          </div>
                        {/if}
                        {#if entry.safety_reasoning}
                          <div>
                            <span class="font-bold text-[var(--color-text-main)]">Safety Reasoning: </span>
                            <span class="text-[var(--color-text-muted)]">{entry.safety_reasoning}</span>
                          </div>
                        {/if}
                        <div>
                          <span class="font-bold text-[var(--color-text-main)]">Full Parameters: </span>
                          <pre class="mt-1 p-2 rounded-lg neu-inset font-mono text-xs overflow-x-auto text-[var(--color-text-main)]">{typeof entry.input === 'object' ? JSON.stringify(entry.input, null, 2) : (entry.input || 'None')}</pre>
                        </div>
                        {#if entry.output_summary || entry.output}
                          <div>
                            <span class="font-bold text-[var(--color-text-main)]">Output Summary: </span>
                            <pre class="mt-1 p-2 rounded-lg neu-inset font-mono text-xs overflow-x-auto text-[var(--color-text-main)]">{entry.output_summary || truncate(entry.output, 500)}</pre>
                          </div>
                        {/if}
                      </div>
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

  {:else if activeTab === 'lessons'}
    {#if lessons.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Safety Lessons</p>
        <p class="text-xs text-[var(--color-text-muted)]">Learned corrective rules from Near Misses and Incidents will appear here.</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each lessons as lesson}
          <div class="neu-card p-5 space-y-3">
            <div class="flex items-center justify-between gap-2">
              <span class="font-mono text-xs font-bold text-[var(--color-primary)]">{lesson.action || 'Safety Policy'}</span>
              <span class="badge-info text-xs px-2 py-0.5 rounded-full font-semibold">
                Confidence: {Math.round((lesson.confidence || 1) * 100)}%
              </span>
            </div>
            <p class="text-sm font-semibold text-[var(--color-text-main)] leading-relaxed">
              {lesson.corrective_rule}
            </p>
            {#if lesson.rationale}
              <p class="text-xs text-[var(--color-text-muted)] leading-relaxed">
                {lesson.rationale}
              </p>
            {/if}
            <div class="pt-2 border-t border-[var(--color-border)] text-xs text-[var(--color-text-muted)] flex justify-between">
              <span>Hits: {lesson.hit_count || 0}</span>
              <span>Updated: {formatTime(lesson.updated_at || lesson.created_at)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if activeTab === 'decisions'}
    {#if decisions.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Decisions Logged</p>
        <p class="text-xs text-[var(--color-text-muted)]">Strategic agent decisions and alternative trade-offs will be tracked here.</p>
      </div>
    {:else}
      <div class="space-y-4">
        {#each decisions as dec}
          <div class="neu-card p-5 space-y-2">
            <div class="flex items-center justify-between">
              <span class="font-bold text-sm text-[var(--color-text-main)]">{dec.decision_title || dec.title || 'Agent Decision'}</span>
              <span class="font-mono text-xs tabular-nums text-[var(--color-text-muted)]">{formatTime(dec.timestamp || dec.created_at)}</span>
            </div>
            <p class="text-xs text-[var(--color-text-muted)] leading-relaxed">{dec.context || dec.description || 'Context not specified'}</p>
            {#if dec.chosen_option}
              <div class="p-2 rounded-lg neu-inset text-xs">
                <span class="font-semibold text-[var(--color-primary)]">Selected Option: </span>
                <span class="text-[var(--color-text-main)]">{dec.chosen_option}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

  {:else if activeTab === 'failures'}
    {#if failures.length === 0}
      <div class="neu-card p-12 text-center">
        <p class="text-base font-semibold text-[var(--color-text-main)] mb-1">No Failure Records</p>
        <p class="text-xs text-[var(--color-text-muted)]">Command faults, timeouts, and sandbox panics will be archived here.</p>
      </div>
    {:else}
      <div class="space-y-4">
        {#each failures as failure}
          <div class="neu-card p-5 space-y-2 border-l-4 border-[var(--color-danger)]">
            <div class="flex items-center justify-between">
              <span class="font-mono text-xs font-bold text-[var(--color-danger)]">{failure.tool || 'System Error'}</span>
              <span class="font-mono text-xs tabular-nums text-[var(--color-text-muted)]">{formatTime(failure.timestamp || failure.created_at)}</span>
            </div>
            <p class="text-xs font-semibold text-[var(--color-text-main)]">{failure.error_message || failure.error || 'Execution failure'}</p>
            {#if failure.remedy}
              <p class="text-xs text-[var(--color-text-muted)]">Remedy: {failure.remedy}</p>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>