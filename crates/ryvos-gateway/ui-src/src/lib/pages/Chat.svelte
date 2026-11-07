<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { apiFetch } from '../api.js';
  import { sendWs, streamingText, resetStreamingText, activityFeed } from '../ws.js';
  import { currentModelInfo } from '../stores.js';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  let { sessionId = '' } = $props();

  let messages = $state([]);
  let messageText = $state('');
  let sending = $state(false);
  let messagesEl = $state(null);
  let currentStreamText = $state('');
  let streaming = $state(false);
  let sessionList = $state([]);
  let allSessions = $state([]);
  let showAllSessions = $state(false);
  let showSidebar = $state(true);
  let sessionSearch = $state('');

  let selectedProvider = $state('gemini');
  let selectedModel = $state('gemini-3.6-flash');
  let keyCount = $state(0);
  let showKeyModal = $state(false);
  let keyPoolInput = $state('');
  let savingKeyPool = $state(false);
  let keyPoolMessage = $state('');

  const providerPresets = {
    gemini: [
      { id: 'gemini-3.6-flash', name: 'Gemini 3.6 Flash (2026 Recommended)' },
      { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash' },
      { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
      { id: 'gemini-2.0-flash', name: 'Gemini 2.0 Flash' }
    ],
    anthropic: [
      { id: 'claude-3-7-sonnet', name: 'Claude 3.7 Sonnet (2026)' },
      { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4' },
      { id: 'claude-opus-4-20250514', name: 'Claude Opus 4' },
      { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet' },
      { id: 'claude-3-5-haiku-20241022', name: 'Claude 3.5 Haiku' }
    ],
    openai: [
      { id: 'gpt-4.5-preview', name: 'GPT-4.5 Preview' },
      { id: 'gpt-4o', name: 'GPT-4o' },
      { id: 'gpt-4o-mini', name: 'GPT-4o Mini' },
      { id: 'o3-mini', name: 'o3-mini' },
      { id: 'o1', name: 'o1' }
    ],
    deepseek: [
      { id: 'deepseek-chat', name: 'DeepSeek-V3' },
      { id: 'deepseek-reasoner', name: 'DeepSeek-R1' }
    ],
    groq: [
      { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B' },
      { id: 'llama-3.1-8b-instant', name: 'Llama 3.1 8B' },
      { id: 'mixtral-8x7b-32768', name: 'Mixtral 8x7B' }
    ],
    openrouter: [
      { id: 'google/gemini-2.5-flash', name: 'OpenRouter: Gemini 2.5 Flash' },
      { id: 'anthropic/claude-3.7-sonnet', name: 'OpenRouter: Claude 3.7 Sonnet' },
      { id: 'deepseek/deepseek-chat', name: 'OpenRouter: DeepSeek-V3' }
    ],
    ollama: [
      { id: 'llama3.2', name: 'Llama 3.2 (Local)' },
      { id: 'qwen2.5:coder', name: 'Qwen 2.5 Coder (Local)' }
    ]
  };

  const providerNames = [
    { id: 'gemini', name: 'Google Gemini' },
    { id: 'anthropic', name: 'Anthropic Claude' },
    { id: 'openai', name: 'OpenAI' },
    { id: 'deepseek', name: 'DeepSeek' },
    { id: 'groq', name: 'Groq' },
    { id: 'openrouter', name: 'OpenRouter' },
    { id: 'ollama', name: 'Ollama (Local)' }
  ];

  marked.setOptions({ breaks: true, gfm: true });

  function renderMarkdown(text) {
    if (!text) return '';
    return DOMPurify.sanitize(marked.parse(text));
  }

  function timeNow() {
    return new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  let unsubStream = () => {};
  let unsubActivity = () => {};
  let unsubModel = () => {};

  async function scrollToBottom() {
    await tick();
    if (messagesEl) {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }
  }

  async function loadSessions() {
    try {
      const data = await apiFetch('/api/sessions');
      const raw = data.sessions || [];
      allSessions = raw;
      const filtered = raw.filter(s => {
        if (showAllSessions) return true;
        const ch = typeof s === 'string' ? '' : (s.channel || '');
        return !ch || ch === 'webui' || ch === 'websocket';
      });
      sessionList = filtered.map(s => typeof s === 'string' ? { id: s, session_key: s } : s);
    } catch (e) {
      console.error('Failed to load sessions:', e);
    }
  }

  async function loadModelConfig() {
    try {
      const data = await apiFetch('/api/model');
      const current = data?.model || data;
      if (current && current.provider) {
        selectedProvider = current.provider;
        if (current.model_id) {
          selectedModel = current.model_id;
        }
        if (typeof current.key_count === 'number') {
          keyCount = current.key_count;
        }
      }
    } catch (e) {
      console.warn('Failed to load model config via HTTP, trying WS:', e);
      try {
        const info = await sendWs('model.get');
        if (info && info.provider) {
          selectedProvider = info.provider;
          selectedModel = info.model_id;
          keyCount = info.key_count || 0;
        }
      } catch {}
    }
  }

  async function applyModelChange(newProvider, newModel) {
    selectedProvider = newProvider;
    selectedModel = newModel;
    try {
      await apiFetch('/api/model', {
        method: 'PUT',
        body: JSON.stringify({ provider: newProvider, model_id: newModel })
      });
      await sendWs('model.set', { provider: newProvider, model_id: newModel });
      currentModelInfo.set({ provider: newProvider, model_id: newModel, key_count: keyCount });
    } catch (e) {
      console.error('Failed to update model config:', e);
    }
  }

  function handleProviderSelect(e) {
    const prov = e.target.value;
    const models = providerPresets[prov] || [];
    const firstModel = models.length > 0 ? models[0].id : '';
    applyModelChange(prov, firstModel);
  }

  function handleModelSelect(e) {
    const mod = e.target.value;
    applyModelChange(selectedProvider, mod);
  }

  async function saveKeyPool() {
    savingKeyPool = true;
    keyPoolMessage = '';
    const raw = keyPoolInput.trim();
    const keys = raw
      .split(/[\n,;]+/)
      .map(k => k.trim())
      .filter(k => k.length > 0);

    try {
      await apiFetch('/api/model', {
        method: 'PUT',
        body: JSON.stringify({
          provider: selectedProvider,
          model_id: selectedModel,
          api_keys: keys
        })
      });
      await sendWs('model.set', {
        provider: selectedProvider,
        model_id: selectedModel,
        api_keys: keys
      });
      keyCount = keys.length;
      keyPoolMessage = `Successfully saved ${keys.length} API keys into rotation pool.`;
      setTimeout(() => {
        showKeyModal = false;
        keyPoolMessage = '';
      }, 1500);
    } catch (e) {
      keyPoolMessage = 'Failed to save keys: ' + (e.message || String(e));
    } finally {
      savingKeyPool = false;
    }
  }

  async function loadHistory() {
    if (!sessionId) return;
    try {
      const data = await apiFetch(`/api/sessions/${encodeURIComponent(sessionId)}/history?limit=100`);
      const raw = data.messages || [];
      messages = raw
        .filter(m => m.text && m.text.trim())
        .map(m => ({
          role: m.role || 'assistant',
          text: m.text || '',
          timestamp: m.timestamp ? new Date(m.timestamp).toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit' }) : '',
        }));
      if (messages.length === 0 && raw.length > 0) {
        messages = [{ role: 'system', text: `${raw.length} messages in this session contain only tool calls or system data.`, timestamp: '' }];
      }
      scrollToBottom();
    } catch (e) {
      console.error('Failed to load history:', e);
      messages = [{ role: 'system', text: 'Failed to load chat history. Check your connection.', timestamp: '', isError: true }];
    }
  }

  async function handleSend() {
    const text = messageText.trim();
    if (!text || sending) return;

    messages = [...messages, { role: 'user', text, timestamp: timeNow() }];
    messageText = '';
    sending = true;
    streaming = true;
    resetStreamingText();
    scrollToBottom();

    let targetSid = sessionId;
    if (!targetSid) {
      targetSid = 'webui:' + Date.now().toString(36);
      window.location.hash = '#/chat/' + encodeURIComponent(targetSid);
    }

    try {
      const res = await sendWs('agent.send', {
        session_id: targetSid,
        message: text,
        provider: selectedProvider,
        model_id: selectedModel
      });

      if (res && res.response && !currentStreamText) {
        messages = [...messages, {
          role: 'assistant',
          text: res.response,
          timestamp: timeNow()
        }];
      }
    } catch (err) {
      messages = [...messages, {
        role: 'assistant',
        text: 'Error: ' + (err.message || String(err)),
        timestamp: timeNow(),
        isError: true
      }];
    } finally {
      sending = false;
      streaming = false;
      resetStreamingText();
      loadSessions();
      scrollToBottom();
    }
  }

  function handleCancel() {
    sendWs('agent.cancel').catch(() => {});
    sending = false;
    streaming = false;
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  function newSession() {
    window.location.hash = '#/chat/webui:' + Date.now().toString(36);
  }

  async function handleApproval(id, approved) {
    try {
      await apiFetch(`/api/approvals/${id}/${approved ? 'approve' : 'deny'}`, { method: 'POST' });
      const msg = messages.find(m => m.type === 'approval' && m.id === id);
      if (msg) {
        msg.resolved = true;
        msg.approved = approved;
        messages = [...messages];
      }
    } catch {}
  }

  onMount(() => {
    unsubStream = streamingText.subscribe(v => {
      currentStreamText = v;
      if (v && !streaming) streaming = true;
    });

    unsubModel = currentModelInfo.subscribe(info => {
      if (info && info.provider) {
        selectedProvider = info.provider;
        if (info.model_id) selectedModel = info.model_id;
        if (typeof info.key_count === 'number') keyCount = info.key_count;
      }
    });

    unsubActivity = activityFeed.subscribe(feed => {
      if (feed.length > 0) {
        const latest = feed[0];

        if (latest.kind === 'tool_start' && streaming) {
          messages = [...messages, {
            type: 'tool_call',
            tool: latest.detail?.replace('Tool: ', '') || 'unknown',
            status: 'running',
            expanded: false,
            input: latest.data?.input || '',
            output: null,
            startTime: Date.now(),
            elapsed: null
          }];
        }

        if (latest.kind === 'tool_end' && streaming) {
          const endedTool = (latest.text || latest.detail?.replace('Tool done: ', '') || '').trim();
          const lastTool = [...messages].reverse().find(
            m => m.type === 'tool_call' && m.status === 'running' && (!endedTool || m.tool === endedTool)
          );
          if (lastTool) {
            lastTool.status = 'done';
            lastTool.output = latest.data?.content || latest.data?.output || latest.detail || '';
            lastTool.elapsed = Date.now() - lastTool.startTime;
            messages = [...messages];
          }
        }

        if (latest.kind === 'approval_requested') {
          messages = [...messages, {
            type: 'approval',
            id: latest.data?.id || latest.data?.request_id || '',
            tool_name: latest.data?.tool_name || 'unknown',
            input_summary: latest.data?.input_summary || '',
            resolved: false,
            approved: false
          }];
        }

        if (latest.kind === 'run_complete' || latest.kind === 'run_error') {
          if (currentStreamText) {
            messages = [...messages, {
              role: 'assistant',
              text: currentStreamText,
              timestamp: timeNow(),
              tokens: latest.data ? {
                input: latest.data.input_tokens || 0,
                output: latest.data.output_tokens || 0
              } : null
            }];
            resetStreamingText();
          }
          sending = false;
          streaming = false;
        }
      }
    });

    loadSessions();
    loadModelConfig();
    loadHistory();
  });

  onDestroy(() => {
    unsubStream();
    unsubModel();
    unsubActivity();
  });

  $effect(() => {
    if (sessionId) {
      loadHistory();
    }
  });

  let currentModelList = $derived(providerPresets[selectedProvider] || []);
  let filteredSessions = $derived(
    sessionSearch.trim()
      ? sessionList.filter(s => {
          const key = (s.session_key || s.id || '').toLowerCase();
          return key.includes(sessionSearch.toLowerCase());
        })
      : sessionList
  );
</script>

<div class="flex gap-4 min-h-[calc(100dvh-8.5rem)] max-w-full">
  {#if showSidebar}
    <aside class="w-64 shrink-0 flex flex-col neu-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-3 shadow-sm hidden md:flex">
      <div class="flex items-center justify-between pb-3 border-b border-[var(--color-border)] mb-3">
        <span class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Conversations</span>
        <button
          type="button"
          onclick={newSession}
          class="btn-primary px-2.5 py-1 text-xs min-h-[32px] rounded-lg"
          title="Start new conversation"
        >
          <span>+ New</span>
        </button>
      </div>

      <div class="mb-3">
        <label for="session-filter-input" class="sr-only">Filter conversations</label>
        <input
          id="session-filter-input"
          type="text"
          bind:value={sessionSearch}
          placeholder="Filter chats..."
          class="neu-inset w-full px-2.5 py-1.5 text-xs placeholder:text-[var(--color-text-secondary)]/60"
        />
      </div>

      <div class="flex items-center gap-1 mb-2 px-1 text-[11px] text-[var(--color-text-secondary)]">
        <label class="flex items-center gap-1.5 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={showAllSessions}
            onchange={loadSessions}
            class="rounded border-[var(--color-border-strong)] text-[var(--color-primary)]"
          />
          <span>Include all channels</span>
        </label>
      </div>

      <div class="flex-1 overflow-y-auto space-y-1 pr-1">
        {#each filteredSessions as s}
          {@const sid = s.session_key || s.id}
          <a
            href="#/chat/{encodeURIComponent(sid)}"
            class="block px-3 py-2 rounded-xl text-xs transition-colors min-h-[36px]
              {sid === sessionId
                ? 'bg-[var(--color-primary-soft)] text-[var(--color-primary)] font-bold shadow-sm'
                : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text)]'}"
          >
            <div class="truncate font-mono">{sid}</div>
            {#if s.last_active}
              <div class="text-[10px] text-[var(--color-text-secondary)] opacity-70">
                {new Date(s.last_active).toLocaleDateString([], { month: 'short', day: 'numeric' })}
              </div>
            {/if}
          </a>
        {/each}
        {#if filteredSessions.length === 0}
          <p class="text-xs text-[var(--color-text-secondary)] text-center py-6">No conversations found</p>
        {/if}
      </div>
    </aside>
  {/if}

  <section class="flex-1 flex flex-col neu-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl shadow-sm overflow-hidden min-w-0">
    <header class="flex flex-wrap items-center gap-2 p-3 border-b border-[var(--color-border)] bg-[var(--color-surface-raised)]">
      <button
        type="button"
        onclick={() => showSidebar = !showSidebar}
        class="btn-soft min-h-[36px] min-w-[36px] p-1.5 rounded-lg text-[var(--color-text)]"
        title="Toggle history sidebar"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
      </button>

      <div class="flex items-center gap-1.5 neu-inset px-2.5 py-1 rounded-xl">
        <label for="provider-select" class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Provider</label>
        <select
          id="provider-select"
          value={selectedProvider}
          onchange={handleProviderSelect}
          class="text-xs font-semibold text-[var(--color-text)] bg-transparent border-0 p-0 focus:ring-0 cursor-pointer"
        >
          {#each providerNames as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </div>

      <div class="flex items-center gap-1.5 neu-inset px-2.5 py-1 rounded-xl">
        <label for="model-select" class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">Model</label>
        <select
          id="model-select"
          value={selectedModel}
          onchange={handleModelSelect}
          class="text-xs font-semibold text-[var(--color-primary)] bg-transparent border-0 p-0 focus:ring-0 cursor-pointer max-w-[180px] truncate"
        >
          {#each currentModelList as m}
            <option value={m.id}>{m.name}</option>
          {/each}
        </select>
      </div>

      <button
        type="button"
        onclick={() => showKeyModal = true}
        class="btn-soft min-h-[36px] px-2.5 py-1 text-xs font-medium text-[var(--color-text)] flex items-center gap-1.5 rounded-xl"
        title="Manage API Key rotation pool"
      >
        <span class="inline-block w-2 h-2 rounded-full {keyCount > 0 ? 'bg-[var(--color-success)]' : 'bg-[var(--color-warning)]'}"></span>
        <span>{keyCount > 0 ? `${keyCount} Keys in Pool` : 'Add API Key'}</span>
      </button>

      <div class="flex-1"></div>

      {#if sending || streaming}
        <button
          type="button"
          onclick={handleCancel}
          class="btn-danger min-h-[36px] px-3 py-1 text-xs rounded-xl flex items-center gap-1.5 font-bold animate-pulse"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <rect x="6" y="6" width="12" height="12" rx="1"/>
          </svg>
          <span>Stop Run</span>
        </button>
      {/if}

      <div class="text-[11px] font-mono text-[var(--color-text-secondary)] hidden sm:block truncate max-w-[160px]">
        {sessionId || 'New Session'}
      </div>
    </header>

    {#if !sessionId && sessionList.length > 0 && messages.length === 0}
      <div class="flex-1 flex items-center justify-center p-6">
        <div class="max-w-md text-center neu-focal p-8">
          <div class="w-12 h-12 rounded-2xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center mx-auto mb-4 neu-card">
            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
          </div>
          <h2 class="text-base font-bold text-[var(--color-text)] mb-1">Ryvos Autonomous Control Plane</h2>
          <p class="text-xs text-[var(--color-text-secondary)] mb-5">Select a prior chat from history, or ask anything to start a fresh autonomous run.</p>
          <div class="grid grid-cols-1 gap-2 text-left">
            <button
              type="button"
              onclick={() => { messageText = "Check runtime health, active tools, and memory invariants."; handleSend(); }}
              class="p-2.5 rounded-xl border border-[var(--color-border)] text-xs font-medium text-[var(--color-text)] hover:border-[var(--color-primary)] hover:bg-[var(--color-primary-soft)] transition-all text-left"
            >
              Check runtime health and memory invariants
            </button>
            <button
              type="button"
              onclick={() => { messageText = "Audit current security policies and list registered MCP tools."; handleSend(); }}
              class="p-2.5 rounded-xl border border-[var(--color-border)] text-xs font-medium text-[var(--color-text)] hover:border-[var(--color-primary)] hover:bg-[var(--color-primary-soft)] transition-all text-left"
            >
              Audit security policies and MCP tools
            </button>
          </div>
        </div>
      </div>
    {:else}
      <div bind:this={messagesEl} class="flex-1 overflow-y-auto p-4 space-y-4">
        {#each messages as msg}
          {#if msg.type === 'tool_call'}
            <div class="max-w-[85%] md:max-w-[75%] mr-auto">
              <button
                type="button"
                onclick={() => { msg.expanded = !msg.expanded; messages = [...messages]; }}
                class="flex items-center gap-2.5 px-3.5 py-2.5 neu-card rounded-xl text-xs text-[var(--color-text)] w-full text-left transition-colors"
              >
                <span class="p-1 rounded-lg bg-[var(--color-primary-soft)] {msg.status === 'running' ? 'text-[var(--color-warning)]' : 'text-[var(--color-success)]'}">
                  <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/></svg>
                </span>
                <span class="font-mono font-medium truncate">{msg.tool}</span>
                <span class="ml-auto text-[11px] font-mono {msg.status === 'running' ? 'text-[var(--color-warning)]' : 'text-[var(--color-success)]'}">
                  {msg.status === 'running' ? 'Executing...' : (msg.elapsed ? msg.elapsed + 'ms' : 'Done')}
                </span>
                <svg class="w-3.5 h-3.5 text-[var(--color-text-secondary)] shrink-0 transition-transform {msg.expanded ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
              </button>
              {#if msg.expanded}
                <div class="mt-1.5 p-3 bg-[#0B1120] rounded-xl text-xs font-mono text-[#EEF3FF] max-h-48 overflow-y-auto border border-[var(--color-border)] shadow-inner">
                  {#if msg.input}
                    <div class="text-[10px] text-[var(--color-text-secondary)] uppercase font-bold mb-1">Input</div>
                    <pre class="whitespace-pre-wrap text-[#EEF3FF] p-0 bg-transparent border-0">{typeof msg.input === 'string' ? msg.input : JSON.stringify(msg.input, null, 2)}</pre>
                  {/if}
                  {#if msg.output}
                    <div class="text-[10px] text-[var(--color-text-secondary)] uppercase font-bold mt-2 mb-1">Output</div>
                    <pre class="whitespace-pre-wrap text-[#EEF3FF] p-0 bg-transparent border-0">{typeof msg.output === 'string' ? msg.output : JSON.stringify(msg.output, null, 2)}</pre>
                  {/if}
                </div>
              {/if}
            </div>
          {:else if msg.type === 'approval'}
            <div class="max-w-[85%] md:max-w-[75%] mr-auto">
              <div class="p-4 bg-[var(--color-surface-raised)] border-2 border-[var(--color-warning)] rounded-2xl shadow-sm">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-4 h-4 text-[var(--color-warning)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                  <span class="text-sm font-bold text-[var(--color-warning)]">Approval Required</span>
                </div>
                <p class="text-xs text-[var(--color-text-secondary)] mb-1">Tool: <span class="font-mono text-[var(--color-text)] font-semibold">{msg.tool_name}</span></p>
                {#if msg.input_summary}
                  <p class="text-xs text-[var(--color-text)] mb-3 font-mono">{msg.input_summary}</p>
                {/if}
                {#if !msg.resolved}
                  <div class="flex gap-2">
                    <button
                      type="button"
                      onclick={() => handleApproval(msg.id, true)}
                      class="btn-primary min-h-[36px] px-4 py-1.5 text-xs font-semibold rounded-lg"
                    >
                      Approve
                    </button>
                    <button
                      type="button"
                      onclick={() => handleApproval(msg.id, false)}
                      class="btn-danger min-h-[36px] px-4 py-1.5 text-xs font-semibold rounded-lg"
                    >
                      Deny
                    </button>
                  </div>
                {:else}
                  <span class="text-xs font-semibold {msg.approved ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">
                    {msg.approved ? 'Approved' : 'Denied'}
                  </span>
                {/if}
              </div>
            </div>
          {:else}
            <div class="max-w-[85%] md:max-w-[75%] {msg.role === 'user' ? 'ml-auto' : 'mr-auto'}">
              <div class="text-[10px] text-[var(--color-text-secondary)] flex items-center gap-2 mb-1 px-1">
                <span class="uppercase tracking-wider font-bold">{msg.role}</span>
                {#if msg.timestamp}
                  <span class="font-mono tabular-nums">{msg.timestamp}</span>
                {/if}
              </div>
              <div
                class="p-4 text-sm leading-relaxed break-words rounded-2xl shadow-sm
                  {msg.role === 'user'
                    ? 'text-white rounded-tr-none'
                    : (msg.isError
                      ? 'bg-[var(--color-surface-raised)] border border-[var(--color-danger)] text-[var(--color-danger)] rounded-tl-none'
                      : 'neu-card rounded-tl-none text-[var(--color-text)]')}"
                style={msg.role === 'user' ? 'background: var(--gradient-primary);' : ''}
              >
                <div class="markdown-content {msg.role === 'user' ? '[&_p]:text-white [&_a]:text-white/90' : ''}">
                  {@html renderMarkdown(msg.text)}
                </div>
              </div>
              {#if msg.tokens}
                <div class="text-[10px] text-[var(--color-text-secondary)] mt-1 font-mono px-1">
                  {msg.tokens.input.toLocaleString()} in / {msg.tokens.output.toLocaleString()} out
                </div>
              {/if}
            </div>
          {/if}
        {/each}

        {#if streaming && currentStreamText}
          <div class="max-w-[85%] md:max-w-[75%] mr-auto">
            <div class="text-[10px] uppercase tracking-wider text-[var(--color-text-secondary)] font-bold mb-1 px-1">assistant</div>
            <div class="p-4 neu-card text-[var(--color-text)] text-sm leading-relaxed break-words rounded-2xl rounded-tl-none shadow-sm">
              <div class="markdown-content">
                {@html renderMarkdown(currentStreamText)}
              </div>
              <span class="streaming-cursor"></span>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <footer class="p-3 border-t border-[var(--color-border)] bg-[var(--color-surface-raised)]">
      <div class="flex gap-2">
        <label for="chat-prompt-input" class="sr-only">Message to agent</label>
        <textarea
          id="chat-prompt-input"
          bind:value={messageText}
          onkeydown={handleKeydown}
          disabled={sending}
          placeholder="Message autonomous agent... (Enter to send, Shift+Enter for newline)"
          rows="1"
          class="neu-inset flex-1 px-4 py-2.5 text-sm resize-none max-h-32 disabled:opacity-50 min-h-[44px]"
        ></textarea>
        {#if sending}
          <button
            type="button"
            onclick={handleCancel}
            class="btn-danger px-4 py-2.5 text-xs font-semibold self-end min-h-[44px]"
          >
            Stop
          </button>
        {:else}
          <button
            type="button"
            onclick={handleSend}
            disabled={!messageText.trim()}
            class="btn-primary px-5 py-2.5 text-sm self-end disabled:opacity-40 min-h-[44px]"
          >
            Send
          </button>
        {/if}
      </div>
    </footer>
  </section>
</div>

{#if showKeyModal}
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Multi-Key Rotation Pool configuration"
    class="fixed inset-0 z-50 bg-[#0F1729]/50 backdrop-blur-sm flex items-center justify-center p-4"
  >
    <div class="neu-focal bg-[var(--color-surface)] border border-[var(--color-border)] rounded-3xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 2l-2 2m-2-2l2 2M3 21l9-9m0 0l3 3m-3-3l-2-2m5-5a5 5 0 11-7.07 7.07"/></svg>
          </div>
          <div>
            <h3 class="text-sm font-bold text-[var(--color-text)] m-0">Multi-Key Rotation Pool</h3>
            <p class="text-[11px] text-[var(--color-text-secondary)] m-0">Provider: {selectedProvider} | Model: {selectedModel}</p>
          </div>
        </div>
        <button
          type="button"
          aria-label="Close dialog"
          onclick={() => showKeyModal = false}
          class="text-[var(--color-text-secondary)] hover:text-[var(--color-text)] text-lg font-bold min-h-[32px] min-w-[32px] flex items-center justify-center"
        >
          &times;
        </button>
      </div>

      <div class="p-3 bg-[var(--color-primary-soft)] border border-[var(--color-border)] rounded-2xl text-xs text-[var(--color-text)] space-y-1">
        <p class="font-semibold text-[var(--color-primary)]">Automatic Key Failover & Load Balancing</p>
        <p class="text-[11px] text-[var(--color-text-secondary)]">Paste multiple API keys (one per line or separated by commas). When any key encounters rate-limit (HTTP 429) or quota errors, Ryvos automatically rotates to the next key without failing your turn.</p>
      </div>

      <div>
        <label for="key-pool-textarea" class="block text-xs font-semibold text-[var(--color-text)] mb-1">API Keys</label>
        <textarea
          id="key-pool-textarea"
          bind:value={keyPoolInput}
          placeholder="Paste API keys here (e.g. key1, key2, ... or one key per line)"
          rows="6"
          class="neu-inset w-full font-mono text-xs px-3 py-2"
        ></textarea>
      </div>

      {#if keyPoolMessage}
        <p class="text-xs font-medium {keyPoolMessage.startsWith('Failed') ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}">
          {keyPoolMessage}
        </p>
      {/if}

      <div class="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={() => showKeyModal = false}
          class="btn-soft min-h-[44px] px-4 py-2 text-xs font-medium text-[var(--color-text)]"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={saveKeyPool}
          disabled={savingKeyPool || !keyPoolInput.trim()}
          class="btn-primary min-h-[44px] px-5 py-2 text-xs font-semibold disabled:opacity-40"
        >
          {savingKeyPool ? 'Saving Pool...' : 'Save & Enable Rotation'}
        </button>
      </div>
    </div>
  </div>
{/if}
