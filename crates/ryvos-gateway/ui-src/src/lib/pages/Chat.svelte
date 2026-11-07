<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { apiFetch } from '../api.js';
  import { sendWs, streamingText, resetStreamingText, activityFeed } from '../ws.js';
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
  let showSessionDropdown = $state(false);

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
      sessionList = filtered.map(s => typeof s === 'string' ? s : (s.id || s.session_key || String(s)));
    } catch (e) {
      console.error('Failed to load sessions:', e);
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
      messages = [{ role: 'system', text: 'Failed to load chat history. Check your connection.', timestamp: '' }];
    }
  }

  function handleSend() {
    const text = messageText.trim();
    if (!text || sending) return;

    messages = [...messages, { role: 'user', text, timestamp: timeNow() }];
    messageText = '';
    sending = true;
    resetStreamingText();
    scrollToBottom();

    let targetSid = sessionId;
    if (!targetSid) {
      targetSid = 'webui:' + Date.now().toString(36);
      window.location.hash = '#/chat/' + encodeURIComponent(targetSid);
    }
    sendWs('agent.send', { session_id: targetSid, message: text });
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
            elapsed: null,
          }];
        }

        if (latest.kind === 'tool_end' && streaming) {
          const endedTool = (latest.text || latest.detail?.replace('Tool done: ', '') || '').trim();
          const lastTool = [...messages].reverse().find(
            m => m.type === 'tool_call' && m.status === 'running' && (!endedTool || m.tool === endedTool)
          );
          if (lastTool) {
            lastTool.status = 'done';
            lastTool.output = latest.data?.output || latest.detail || '';
            lastTool.elapsed = Date.now() - lastTool.startTime;
            messages = [...messages];
          }
        }

        if (latest.kind === 'approval_requested') {
          messages = [...messages, {
            type: 'approval',
            id: latest.data?.request_id || '',
            tool_name: latest.data?.tool_name || 'unknown',
            input_summary: latest.data?.input_summary || '',
            resolved: false,
            approved: false,
          }];
        }

        if (streaming && (latest.kind === 'run_complete' || latest.kind === 'run_error')) {
          if (currentStreamText) {
            messages = [...messages, {
              role: 'assistant',
              text: currentStreamText,
              timestamp: timeNow(),
              tokens: latest.data ? {
                input: latest.data.input_tokens || 0,
                output: latest.data.output_tokens || 0,
              } : null,
            }];
          }
          if (latest.kind === 'run_error') {
            messages = [...messages, { role: 'assistant', text: 'Error: ' + (latest.data?.error || 'Unknown'), timestamp: timeNow() }];
          }
          streaming = false;
          sending = false;
          resetStreamingText();
        }
      }
    });

    loadSessions();
    if (sessionId) loadHistory();
  });

  onDestroy(() => {
    unsubStream();
    unsubActivity();
  });

  $effect(() => {
    if (sessionId) {
      messages = [];
      loadHistory();
    }
  });

  $effect(() => {
    if (currentStreamText) {
      scrollToBottom();
    }
  });
</script>

<div class="flex flex-col h-[calc(100vh-6.5rem)] md:h-[calc(100vh-4rem)]">
  <div class="flex items-center gap-3 pb-3 border-b border-[#DCE4F3] mb-3">
    <div class="relative">
      <button
        type="button"
        onclick={() => { showSessionDropdown = !showSessionDropdown; loadSessions(); }}
        class="btn-surface flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-[#1B2943]"
      >
        <svg class="w-3.5 h-3.5 text-[#2448CB]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        <span class="max-w-[140px] md:max-w-[200px] truncate">{sessionId || 'Select session'}</span>
        <svg class="w-3 h-3 text-[#778399] transition-transform {showSessionDropdown ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
      </button>

      {#if showSessionDropdown}
        <div class="absolute top-full left-0 mt-1.5 w-72 bg-white border border-[#DCE4F3] rounded-2xl shadow-soft-float z-30 max-h-60 overflow-y-auto p-1 space-y-0.5">
          {#each sessionList as sid}
            <a
              href="#/chat/{encodeURIComponent(sid)}"
              onclick={() => showSessionDropdown = false}
              class="block px-3 py-2 text-xs font-mono rounded-xl transition-colors truncate {sid === sessionId ? 'bg-[#E8EEFF] text-[#2448CB] font-semibold' : 'text-[#64748B] hover:bg-[#F4F6FF] hover:text-[#1B2943]'}"
            >
              {sid}
            </a>
          {/each}
          {#if sessionList.length === 0}
            <p class="px-3 py-3 text-xs text-[#778399] text-center">No active sessions</p>
          {/if}
        </div>
      {/if}
    </div>

    <label class="flex items-center gap-2 cursor-pointer select-none text-xs text-[#64748B]">
      <input
        type="checkbox"
        bind:checked={showAllSessions}
        onchange={() => loadSessions()}
        class="rounded border-[#DCE4F3] text-[#2448CB] focus:ring-[#2448CB]"
      />
      <span class="hidden sm:inline">All channels</span>
    </label>

    <div class="flex-1"></div>

    <button
      type="button"
      onclick={newSession}
      class="btn-surface px-3 py-1.5 text-xs font-semibold text-[#2448CB] flex items-center gap-1.5 hover:bg-[#E8EEFF]"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      <span>New</span>
    </button>
  </div>

  {#if !sessionId && sessionList.length > 0}
    <div class="flex-1 overflow-y-auto p-2">
      <div class="max-w-xl mx-auto space-y-2">
        <h3 class="text-sm font-semibold text-[#778399] mb-3">Select a conversation or start new</h3>
        {#each sessionList as sid}
          <a
            href="#/chat/{encodeURIComponent(sid)}"
            class="card-soft block p-4 text-sm font-mono text-[#1B2943] hover:border-[#2448CB] hover:shadow-soft-float transition-all truncate"
          >
            {sid}
          </a>
        {/each}
      </div>
    </div>
  {:else if !sessionId}
    <div class="flex-1 flex items-center justify-center p-4">
      <div class="text-center card-soft p-8 max-w-sm">
        <div class="w-12 h-12 rounded-2xl bg-[#E8EEFF] text-[#2448CB] flex items-center justify-center mx-auto mb-4">
          <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/></svg>
        </div>
        <h3 class="text-lg font-heading font-bold text-[#1B2943] mb-1">No Active Session</h3>
        <p class="text-[#778399] text-xs mb-5">Start a fresh conversation turn with the autonomous agent runtime.</p>
        <button
          type="button"
          onclick={newSession}
          class="btn-primary w-full py-2.5 text-sm"
        >
          Start New Session
        </button>
      </div>
    </div>
  {:else}
    <div bind:this={messagesEl} class="flex-1 overflow-y-auto py-3 px-1 space-y-4">
      {#each messages as msg}
        {#if msg.type === 'tool_call'}
          <div class="max-w-[85%] md:max-w-[75%] mr-auto">
            <button
              type="button"
              onclick={() => { msg.expanded = !msg.expanded; messages = [...messages]; }}
              class="flex items-center gap-2.5 px-3.5 py-2.5 bg-white border border-[#DCE4F3] rounded-xl text-xs text-[#1B2943] shadow-soft-sm hover:border-[#2448CB] w-full text-left transition-colors"
            >
              <span class="p-1 rounded-lg bg-[#E8EEFF] {msg.status === 'running' ? 'text-[#C58A2A]' : 'text-[#219B70]'}">
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/></svg>
              </span>
              <span class="font-mono font-medium truncate">{msg.tool}</span>
              <span class="ml-auto text-[11px] font-mono {msg.status === 'running' ? 'text-[#C58A2A]' : 'text-[#219B70]'}">
                {msg.status === 'running' ? 'Running...' : (msg.elapsed ? msg.elapsed + 'ms' : 'Done')}
              </span>
              <svg class="w-3.5 h-3.5 text-[#778399] shrink-0 transition-transform {msg.expanded ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
            </button>
            {#if msg.expanded}
              <div class="mt-1.5 p-3 bg-[#0F1729] rounded-xl text-xs font-mono text-[#E2E8F0] max-h-48 overflow-y-auto border border-[#DCE4F3]/20 shadow-soft-inset">
                {#if msg.input}
                  <div class="text-[10px] text-[#778399] uppercase font-bold mb-1">Input</div>
                  <pre class="whitespace-pre-wrap text-[#E2E8F0] p-0 bg-transparent border-0">{typeof msg.input === 'string' ? msg.input : JSON.stringify(msg.input, null, 2)}</pre>
                {/if}
                {#if msg.output}
                  <div class="text-[10px] text-[#778399] uppercase font-bold mt-2 mb-1">Output</div>
                  <pre class="whitespace-pre-wrap text-[#E2E8F0] p-0 bg-transparent border-0">{typeof msg.output === 'string' ? msg.output : JSON.stringify(msg.output, null, 2)}</pre>
                {/if}
              </div>
            {/if}
          </div>
        {:else if msg.type === 'approval'}
          <div class="max-w-[85%] md:max-w-[75%] mr-auto">
            <div class="p-4 bg-[#FFFBEB] border border-[#C58A2A] rounded-2xl shadow-soft-sm">
              <div class="flex items-center gap-2 mb-2">
                <svg class="w-4 h-4 text-[#C58A2A]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                <span class="text-sm font-bold text-[#C58A2A]">Approval Required</span>
              </div>
              <p class="text-xs text-[#64748B] mb-1">Tool: <span class="font-mono text-[#1B2943] font-semibold">{msg.tool_name}</span></p>
              {#if msg.input_summary}
                <p class="text-xs text-[#64748B] mb-3">{msg.input_summary}</p>
              {/if}
              {#if !msg.resolved}
                <div class="flex gap-2">
                  <button
                    type="button"
                    onclick={() => handleApproval(msg.id, true)}
                    class="px-4 py-1.5 bg-[#219B70] hover:bg-[#1A825D] text-white rounded-lg text-xs font-semibold shadow-sm transition-colors"
                  >
                    Approve
                  </button>
                  <button
                    type="button"
                    onclick={() => handleApproval(msg.id, false)}
                    class="px-4 py-1.5 bg-[#D94B5B] hover:bg-[#BF3E4C] text-white rounded-lg text-xs font-semibold shadow-sm transition-colors"
                  >
                    Deny
                  </button>
                </div>
              {:else}
                <span class="text-xs font-semibold {msg.approved ? 'text-[#219B70]' : 'text-[#D94B5B]'}">
                  {msg.approved ? 'Approved' : 'Denied'}
                </span>
              {/if}
            </div>
          </div>
        {:else}
          <div class="max-w-[85%] md:max-w-[75%] {msg.role === 'user' ? 'ml-auto' : 'mr-auto'}">
            <div class="text-[10px] text-[#778399] flex items-center gap-2 mb-1 px-1">
              <span class="uppercase tracking-wider font-bold">{msg.role}</span>
              {#if msg.timestamp}
                <span class="font-mono tabular-nums">{msg.timestamp}</span>
              {/if}
            </div>
            <div class="p-4 text-sm leading-relaxed break-words rounded-2xl {msg.role === 'user' ? 'bg-[#2448CB] text-white rounded-tr-none shadow-soft-sm' : 'bg-white border border-[#DCE4F3] text-[#1B2943] rounded-tl-none shadow-soft'}">
              <div class="markdown-content {msg.role === 'user' ? '[&_p]:text-white [&_a]:text-white/90' : ''}">
                {@html renderMarkdown(msg.text)}
              </div>
            </div>
            {#if msg.tokens}
              <div class="text-[10px] text-[#778399] mt-1 font-mono px-1">
                {msg.tokens.input.toLocaleString()} in / {msg.tokens.output.toLocaleString()} out
              </div>
            {/if}
          </div>
        {/if}
      {/each}

      {#if streaming && currentStreamText}
        <div class="max-w-[85%] md:max-w-[75%] mr-auto">
          <div class="text-[10px] uppercase tracking-wider text-[#778399] font-bold mb-1 px-1">assistant</div>
          <div class="p-4 bg-white border border-[#DCE4F3] text-[#1B2943] text-sm leading-relaxed break-words rounded-2xl rounded-tl-none shadow-soft">
            <div class="markdown-content">
              {@html renderMarkdown(currentStreamText)}
            </div>
            <span class="streaming-cursor"></span>
          </div>
        </div>
      {/if}
    </div>

    <div class="flex gap-2 pt-3 border-t border-[#DCE4F3] mt-2">
      <textarea
        bind:value={messageText}
        onkeydown={handleKeydown}
        disabled={sending}
        placeholder="Type a message..."
        rows="1"
        class="input-soft flex-1 px-4 py-2.5 text-sm resize-none max-h-28 disabled:opacity-50"
      ></textarea>
      <button
        type="button"
        onclick={handleSend}
        disabled={sending || !messageText.trim()}
        class="btn-primary px-5 py-2.5 text-sm self-end disabled:opacity-40"
      >
        Send
      </button>
    </div>
  {/if}
</div>
