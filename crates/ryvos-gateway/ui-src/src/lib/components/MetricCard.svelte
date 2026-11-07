<script>
  let { label = '', value = '', icon = '', description = '' } = $props();
</script>

<div class="neu-card bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl p-5 shadow-sm transition-all duration-180 hover:border-[var(--color-border-strong)]">
  <div class="flex items-start justify-between">
    <div>
      <p class="text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">{label}</p>
      <p class="mt-1.5 text-2xl sm:text-3xl font-bold text-[var(--color-text)] tabular-nums tracking-tight">{value}</p>
      {#if description}
        <p class="mt-1 text-xs text-[var(--color-text-secondary)]">{description}</p>
      {/if}
    </div>
    <div class="rounded-xl bg-[var(--color-primary-soft)] text-[var(--color-primary)] p-2.5 shadow-sm">
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        {@html icon}
      </svg>
    </div>
  </div>
</div>
