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

<div
  role="dialog"
  aria-modal="true"
  aria-label="Connect to Ryvos Control Plane"
  class="fixed inset-0 bg-[#0F1729]/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
>
  <div class="neu-focal bg-[var(--color-surface)] border border-[var(--color-border)] rounded-3xl p-8 w-[420px] max-w-full text-center shadow-2xl">
    <div class="mb-5 flex justify-center">
      <div class="w-16 h-16 rounded-2xl bg-[var(--color-primary-soft)] flex items-center justify-center neu-card">
        <img src={logoUrl} alt="Ryvos logo" class="w-10 h-10 object-contain" />
      </div>
    </div>

    <h2 class="text-xl font-bold text-[var(--color-text)] tracking-tight mb-1">
      Ryvos Control Plane
    </h2>
    <p class="text-[var(--color-text-secondary)] text-sm mb-6">
      Enter your gateway authentication token to connect
    </p>

    <div class="mb-3 text-left">
      <label for="api-key-input" class="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)] mb-1.5">
        API Key or Gateway Token
      </label>
      <input
        id="api-key-input"
        type="password"
        bind:value={apiKeyValue}
        onkeydown={handleKeydown}
        placeholder="rk_..."
        class="neu-inset w-full px-3.5 py-2.5 font-mono text-sm placeholder:text-[var(--color-text-secondary)]/60 min-h-[44px]"
      />
    </div>

    <p class="text-xs text-[var(--color-text-secondary)] mb-6 text-left">
      Leave blank if no gateway authentication is required on this cluster instance
    </p>

    <button
      type="button"
      onclick={handleSubmit}
      class="btn-primary w-full py-3 text-sm flex items-center justify-center gap-2 min-h-[44px]"
    >
      <span>Connect to Control Plane</span>
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/>
      </svg>
    </button>
  </div>
</div>
