<script lang="ts">
  let { visible = false, onRate = (_quality: number) => {} } = $props();

  const buttons = [
    { quality: 1, label: "Wiederholen", key: "1", bg: "bg-accent-incorrect dark:bg-accent-incorrect-dark", shadow: "shadow-red-500/25" },
    { quality: 2, label: "Schwer",       key: "2", bg: "bg-accent-hard dark:bg-accent-hard-dark",           shadow: "shadow-orange-500/25" },
    { quality: 3, label: "Okay",         key: "3", bg: "bg-accent-correct dark:bg-accent-correct-dark",     shadow: "shadow-amber-500/25" },
    { quality: 4, label: "Gut",          key: "4", bg: "bg-accent-easy dark:bg-accent-easy-dark",           shadow: "shadow-green-500/25" },
  ];
</script>

{#if visible}
  <div class="flex gap-3 justify-center animate-in fade-in slide-in-from-bottom-4 duration-200">
    {#each buttons as btn}
      <button
        onclick={() => onRate(btn.quality)}
        class="score-button {btn.bg} {btn.shadow} text-white px-5 py-3 font-semibold flex flex-col items-center gap-0.5 min-w-24"
        title="Taste {btn.key}"
      >
        <span class="text-lg leading-none">{btn.key}</span>
        <span class="text-xs opacity-90">{btn.label}</span>
      </button>
    {/each}
  </div>
{/if}

<!-- Hint -->
{#if !visible}
  <div class="flex items-center justify-center gap-2 text-sm text-secondary mt-6">
    <kbd class="px-2 py-0.5 bg-white/10 rounded text-xs font-mono">Leertaste</kbd>
    <span>oder klicken zum Umdrehen</span>
  </div>
{/if}
