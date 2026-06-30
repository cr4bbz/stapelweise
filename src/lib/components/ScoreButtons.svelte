<script lang="ts">
  let { visible = false, onRate = (_quality: number) => {} } = $props();

  const buttons = [
    { quality: 1, label: "Wiederholen", key: "1", color: "bg-[#DC2626] hover:bg-[#B91C1C]" },
    { quality: 2, label: "Schwer", key: "2", color: "bg-[#F97316] hover:bg-[#EA580C]" },
    { quality: 3, label: "Okay", key: "3", color: "bg-[#E6A817] hover:bg-[#CA9314]" },
    { quality: 4, label: "Gut", key: "4", color: "bg-[#22C55E] hover:bg-[#16A34A]" },
  ];
</script>

{#if visible}
  <div class="flex gap-3 justify-center animate-in fade-in slide-in-from-bottom-4 duration-200">
    {#each buttons as btn}
      <button
        onclick={() => onRate(btn.quality)}
        class="rounded-button {btn.color} text-white px-5 py-3 font-medium hover:scale-105 transition-all flex flex-col items-center gap-1 min-w-24"
        title="Taste {btn.key}"
      >
        <span class="text-lg font-bold">{btn.key}</span>
        <span class="text-xs">{btn.label}</span>
      </button>
    {/each}
  </div>
{/if}

<!-- Hint -->
{#if !visible}
  <div class="flex items-center justify-center gap-2 text-sm text-secondary mt-6 animate-pulse">
    <kbd class="px-2 py-0.5 bg-white/20 rounded text-xs font-mono">Leertaste</kbd>
    <span>oder klicken zum Umdrehen</span>
  </div>
{/if}
