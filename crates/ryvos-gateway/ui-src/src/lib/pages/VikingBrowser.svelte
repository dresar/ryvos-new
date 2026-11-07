<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let tree = $state([]);
  let loading = $state(true);
  let error = $state('');
  let expandedPaths = $state({});
  let selectedContent = $state(null);
  let selectedPath = $state('');
  let selectedLevel = $state('L1');
  let contentLoading = $state(false);
  let searchQuery = $state('');
  let searchResults = $state([]);
  let searchLoading = $state(false);

  async function loadTree(path = 'viking://') {
    try {
      const data = await apiFetch('/api/viking/list?path=' + encodeURIComponent(path));
      return Array.isArray(data) ? data : (data.entries || data.items || data.children || []);
    } catch {
      return [];
    }
  }

  onMount(async () => {
    try {
      tree = await loadTree('viking://');
    } catch (e) {
      error = e.message || 'Failed to load memory context';
    } finally {
      loading = false;
    }
  });

  async function toggleDir(item) {
    const path = item.path || item.name;
    if (expandedPaths[path]) {
      const next = { ...expandedPaths };
      delete next[path];
      expandedPaths = next;
    } else {
      const children = await loadTree(path);
      expandedPaths = { ...expandedPaths, [path]: children };
    }
  }

  async function readLeaf(path, level = selectedLevel) {
    selectedPath = path;
    selectedLevel = level;
    contentLoading = true;
    selectedContent = null;
    try {
      const data = await apiFetch('/api/viking/read?path=' + encodeURIComponent(path) + '&level=' + level);
      selectedContent = data;
    } catch (e) {
      selectedContent = { error: e.message || 'Failed to read context' };
    } finally {
      contentLoading = false;
    }
  }

  async function handleSearch() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }
    searchLoading = true;
    try {
      const data = await apiFetch('/api/viking/search?q=' + encodeURIComponent(searchQuery));
      searchResults = data.results || data.items || data || [];
    } catch (e) {
      searchResults = [];
    } finally {
      searchLoading = false;
    }
  }

  function isDirectory(item) {
    return item.is_directory === true || item.type === 'directory' || item.type === 'dir';
  }

  function getDisplayName(item) {
    const name = item.name || item.path || '';
    return name.split('/').filter(Boolean).pop() || name;
  }
</script>

{#snippet treeNode(items, depth)}
  {#each items as item}
    {@const path = item.path || item.name}
    {#if isDirectory(item)}
      <button
        type="button"
        onclick={() => toggleDir(item)}
        class="w-full flex items-center gap-2 px-3 py-2 text-left text-sm rounded-xl min-h-[44px] transition-colors hover:bg-[var(--color-bg)] {depth === 0 ? 'text-[var(--color-text-main)] font-semibold' : 'text-[var(--color-text-muted)]'}"
      >
        <svg class="w-4 h-4 shrink-0 text-[var(--color-text-muted)] transition-transform {expandedPaths[path] ? 'rotate-90' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
        <svg class="w-4 h-4 shrink-0 text-[var(--color-primary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        <span class="truncate">{getDisplayName(item)}</span>
      </button>
      {#if expandedPaths[path]}
        <div class="ml-5 border-l border-[var(--color-border)] pl-2 space-y-1 mt-1">
          {@render treeNode(expandedPaths[path], depth + 1)}
        </div>
      {/if}
    {:else}
      <button
        type="button"
        onclick={() => readLeaf(path)}
        class="w-full flex items-center gap-2 px-3 py-2 text-left text-sm rounded-xl min-h-[44px] transition-colors {selectedPath === path ? 'neu-inset text-[var(--color-primary)] font-bold' : 'text-[var(--color-text-main)] hover:bg-[var(--color-bg)]'}"
      >
        <svg class="w-4 h-4 shrink-0 text-[var(--color-text-muted)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/>
        </svg>
        <span class="truncate">{getDisplayName(item)}</span>
      </button>
    {/if}
  {/each}
{/snippet}

<div class="space-y-6">
  <div>
    <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-main)]">Memory and Context Knowledge Browser</h2>
    <p class="text-sm text-[var(--color-text-muted)] mt-1">Hierarchical tree inspection and semantic search across persistent Viking memory artifacts</p>
  </div>

  <div class="neu-card p-4 flex flex-wrap items-center gap-3">
    <label for="viking-search-input" class="text-xs font-semibold text-[var(--color-text-muted)]">Search Artifacts:</label>
    <input
      id="viking-search-input"
      type="text"
      bind:value={searchQuery}
      placeholder="Search memory records or embeddings..."
      onkeydown={(e) => { if (e.key === 'Enter') handleSearch(); }}
      class="neu-inset min-h-[44px] px-4 py-2 text-sm text-[var(--color-text-main)] rounded-xl flex-1 min-w-[240px]"
    />
    <button
      type="button"
      onclick={handleSearch}
      disabled={searchLoading}
      class="btn-primary min-h-[44px] px-6 py-2 text-xs font-semibold rounded-xl"
    >
      {searchLoading ? 'Searching...' : 'Search'}
    </button>
    {#if searchQuery}
      <button
        type="button"
        onclick={() => { searchQuery = ''; searchResults = []; }}
        class="btn-soft min-h-[44px] px-4 py-2 text-xs font-semibold rounded-xl"
      >
        Clear
      </button>
    {/if}
  </div>

  {#if searchResults.length > 0}
    <div class="neu-card p-4 space-y-2">
      <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-primary)]">Search Results ({searchResults.length})</h3>
      <div class="divide-y divide-[var(--color-border)]">
        {#each searchResults as res}
          {@const p = res.path || res.id || res.name}
          <button
            type="button"
            onclick={() => readLeaf(p)}
            class="w-full text-left py-2 px-3 rounded-lg hover:bg-[var(--color-bg)] transition-colors flex items-center justify-between"
          >
            <span class="font-mono text-xs text-[var(--color-text-main)] truncate">{p}</span>
            <span class="text-xs text-[var(--color-primary)] font-semibold">Inspect</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="neu-card p-12 text-center">
      <div class="inline-block w-8 h-8 border-4 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin mb-3"></div>
      <p class="text-sm text-[var(--color-text-muted)]">Loading memory hierarchy...</p>
    </div>
  {:else if error}
    <div class="neu-card p-8 border-l-4 border-[var(--color-danger)] text-left">
      <h3 class="text-base font-bold text-[var(--color-danger)] mb-1">Memory Service Offline</h3>
      <p class="text-sm text-[var(--color-text-muted)]">{error}</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <div class="neu-card p-5 max-h-[640px] overflow-y-auto">
        <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-3">Context Hierarchy</h3>
        {#if tree.length === 0}
          <p class="text-sm text-[var(--color-text-muted)] text-center py-12">No memory items recorded</p>
        {:else}
          <div class="space-y-1">
            {@render treeNode(tree, 0)}
          </div>
        {/if}
      </div>

      <div class="neu-card p-5 max-h-[640px] overflow-y-auto flex flex-col">
        <div class="flex items-center justify-between pb-3 mb-3 border-b border-[var(--color-border)]">
          <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] truncate max-w-xs">
            {selectedPath || 'Memory Payload Inspector'}
          </h3>
          {#if selectedPath}
            <div class="flex items-center gap-1">
              {#each ['L0', 'L1', 'L2'] as lvl}
                <button
                  type="button"
                  onclick={() => readLeaf(selectedPath, lvl)}
                  class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all {selectedLevel === lvl ? 'btn-primary' : 'btn-soft text-[var(--color-text-muted)]'}"
                >
                  {lvl}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#if contentLoading}
          <div class="py-16 text-center text-sm text-[var(--color-text-muted)]">
            Loading context payload...
          </div>
        {:else if selectedContent}
          {#if selectedContent.error}
            <div class="p-4 rounded-xl bg-red-500/10 text-xs font-mono text-[var(--color-danger)]">
              {selectedContent.error}
            </div>
          {:else}
            <pre class="neu-inset p-4 rounded-xl text-xs font-mono text-[var(--color-text-main)] whitespace-pre-wrap overflow-x-auto flex-1">{JSON.stringify(selectedContent, null, 2)}</pre>
          {/if}
        {:else}
          <div class="py-16 text-center text-sm text-[var(--color-text-muted)]">
            Select an artifact from the hierarchy to inspect payload and resolution levels.
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>