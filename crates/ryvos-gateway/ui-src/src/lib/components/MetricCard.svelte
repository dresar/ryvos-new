<script>
  let { label = '', value = '', icon = '', description = '' } = $props();
</script>

<div class="bg-white border border-[#DCE4F3] rounded-2xl shadow-soft p-5 transition-all duration-200 hover:shadow-soft-float">
  <div class="flex items-start justify-between">
    <div>
      <p class="label">{label}</p>
      <p class="mt-1 text-3xl font-heading font-bold text-[#1B2943] tabular-nums tracking-tight">{value}</p>
      {#if description}
        <p class="mt-1 text-sm text-[#778399]">{description}</p>
      {/if}
    </div>
    <div class="rounded-xl bg-[#E8EEFF] text-[#2448CB] p-2.5 shadow-sm">
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        {@html icon}
      </svg>
    </div>
  </div>
</div>
