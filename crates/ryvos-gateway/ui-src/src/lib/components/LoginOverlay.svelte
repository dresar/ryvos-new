<script>
  import logoUrl from '../../assets/logo.png';

  let { onlogin = () => {} } = $props();
  let apiKeyValue = $state('');

  function handleSubmit() {
    onlogin({ key: apiKeyValue.trim() });
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') handleSubmit();
  }
</script>

<div class="fixed inset-0 bg-[#0F1729]/40 backdrop-blur-sm flex items-center justify-center z-50 p-4">
  <div class="bg-white border border-[#DCE4F3] rounded-3xl shadow-soft-float p-8 w-[380px] max-w-full text-center">
    <div class="mb-5 flex justify-center">
      <div class="w-16 h-16 rounded-2xl bg-[#E8EEFF] flex items-center justify-center shadow-soft-sm">
        <img src={logoUrl} alt="Ryvos" class="w-10 h-10 object-contain" />
      </div>
    </div>

    <h2 class="text-xl font-heading font-bold text-[#1B2943] tracking-tight mb-1">Welcome to Ryvos</h2>
    <p class="text-[#64748B] text-sm mb-6">Enter your API key to access the gateway dashboard</p>

    <div class="mb-2 text-left">
      <label for="api-key-input" class="label block mb-1.5">API Key</label>
      <input
        id="api-key-input"
        type="password"
        bind:value={apiKeyValue}
        onkeydown={handleKeydown}
        placeholder="rk_..."
        class="input-soft w-full px-3.5 py-2.5 font-mono text-sm placeholder:text-[#778399]"
      />
    </div>

    <p class="text-xs text-[#778399] mb-6 text-left">Leave blank if no auth is configured on this instance</p>

    <button
      type="button"
      onclick={handleSubmit}
      class="btn-primary w-full py-3 text-sm flex items-center justify-center gap-2"
    >
      <span>Connect</span>
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/>
      </svg>
    </button>
  </div>
</div>
