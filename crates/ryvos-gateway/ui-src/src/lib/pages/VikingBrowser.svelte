<script>
  import { onMount } from 'svelte';
  import { apiFetch } from '../api.js';

  let tree = $state([]);
  let loading = $state(true);
  let error = $state('');
  let expandedPaths = $state({});
  let selectedContent = $state(null);
  let selectedPath = $state('');
  let contentLoading = $state(false);

  async function loadTree(path = 'viking://') {
    try {
      const data = await apiFetch(`/api/viking/list?path=${encodeURIComponent(path)}`);
      return Array.isArray(data) ? data : (data.entries || data.items || data.children || []);
    } catch (e) {
      return [];
    }
  }

  onMount(async () => {
    try {
      tree = await loadTree('viking://');
    } catch (e) {
      error = e.message;
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

  async function readLeaf(item) {
    const path = item.path || item.name;
    selectedPath = path;
    contentLoading = true;
    selectedContent = null;
    try {
      const data = await apiFetch(`/api/viking/read?path=${encodeURIComponent(path)}&level=L1`);
      selectedContent = data;
    } catch (e) {
      selectedContent = { error: e.message };
    } finally {
      contentLoading = false;
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
    {#if isDirectory(item)}
      <button
        type="button"
        onclick={() => toggleDir(item)}
        class="w-full flex items-center gap-2 px-3 text-left text-sm rounded-xl hover:bg-[#F4F6FF] transition-colors duration-150
               {depth === 0 ? 'py-2 text-[#1B2943] font-medium' : 'py-1.5 text-[#64748B]'}"
      >
        <svg class="w-3.5 h-3.5 shrink-0 text-[#778399] transition-transform duration-150 {expandedPaths[item.path || item.name] ? 'rotate-90' : ''}"
          viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
        <svg class="w-4 h-4 shrink-0 text-[#2448CB]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        <span class="truncate">{getDisplayName(item)}</span>
      </button>
      {#if expandedPaths[item.path || item.name]}
        <div class="ml-5 border-l border-[#DCE4F3] pl-2.5 space-y-0.5 mt-0.5">
          {@render treeNode(expandedPaths[item.path || item.name], depth + 1)}
        </div>
      {/if}
    {:else}
      <button
        type="button"
        onclick={() => readLeaf(item)}
        class="w-full flex items-center gap-2 px-3 text-left text-sm rounded-xl transition-colors duration-150
               {depth === 0 ? 'py-2' : 'py-1.5'}
               {selectedPath === (item.path || item.name)
                 ? 'bg-[#E8EEFF] text-[#2448CB] font-semibold'
                 : (depth === 0 ? 'text-[#1B2943]' : 'text-[#64748B]') + ' hover:bg-[#F4F6FF]'}"
      >
        <svg class="w-3.5 h-3.5 shrink-0 text-[#778399]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/>
        </svg>
        <span class="truncate">{getDisplayName(item)}</span>
      </button>
    {/if}
  {/each}
{/snippet}

<div>
  <div class="mb-6">
    <h2 class="text-2xl font-heading font-bold tracking-tight text-[#1B2943]">Memory & Context Browser</h2>
    <p class="text-[#778399] text-sm mt-1">Explore structured context trees and knowledge artifacts</p>
  </div>

  {#if loading}
    <div class="card-soft p-8 text-center">
      <p class="text-[#778399] text-sm animate-pulse">Loading memory context tree...</p>
    </div>
  {:else if error}
    <div class="card-soft p-12 text-center">
      <p class="text-[#778399] text-sm">Memory service not available: {error}</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <div class="card-soft p-5 max-h-[600px] overflow-y-auto">
        <h3 class="text-sm font-bold text-[#1B2943] mb-4">Context Hierarchy</h3>
        {#if tree.length === 0}
          <p class="text-[#778399] text-sm text-center py-8">No context entries recorded at root</p>
        {:else}
          <div class="space-y-0.5">
            {@render treeNode(tree, 0)}
          </div>
        {/if}
      </div>

      <div class="card-soft p-5 max-h-[600px] overflow-y-auto">
        <h3 class="text-sm font-bold text-[#1B2943] mb-4">
          {#if selectedPath}
            <span class="font-mono text-xs text-[#2448CB] bg-[#E8EEFF] px-2 py-0.5 rounded-md truncate block">{selectedPath}</span>
          {:else}
            Context Content
          {/if}
        </h3>

        {#if contentLoading}
          <p class="text-[#778399] text-sm text-center py-8 animate-pulse">Loading memory payload...</p>
        {:else if selectedContent}
          {#if selectedContent.error}
            <p class="text-[#D94B5B] text-sm">{selectedContent.error}</p>
          {:else}
            <pre class="bg-[#0F1729] text-[#E2E8F0] rounded-xl p-4 text-xs font-mono whitespace-pre-wrap overflow-x-auto shadow-soft-inset">{JSON.stringify(selectedContent, null, 2)}</pre>
          {/if}
        {:else}
          <p class="text-[#778399] text-sm text-center py-8">Select an artifact from the tree to inspect details</p>
        {/if}
      </div>
    </div>
  {/if}
</div>
