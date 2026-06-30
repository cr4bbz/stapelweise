<script lang="ts">
  let { current = 0, total = 0 } = $props();

  let pct = $derived(total > 0 ? (current / total) * 100 : 0);
</script>

<div class="w-full">
  <div class="flex justify-between text-xs text-secondary mb-1">
    <span>{current} / {total}</span>
    <span>{Math.round(pct)}%</span>
  </div>
  <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
    <div
      class="h-full bg-accent-correct rounded-full transition-all duration-500 ease-out"
      style="width: {pct}%"
    ></div>
  </div>
</div>
