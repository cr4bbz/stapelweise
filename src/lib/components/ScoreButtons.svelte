<script lang="ts">
  let { enabled = false, animate = true, smooth = true, onRate = (_quality: number) => {} } = $props();

  const buttons = [
    { quality: 1, label: "Wiederholen", key: "1", bg: "bg-accent-incorrect dark:bg-accent-incorrect-dark", shadow: "shadow-red-500/25" },
    { quality: 2, label: "Schwer",       key: "2", bg: "bg-accent-hard dark:bg-accent-hard-dark",           shadow: "shadow-orange-500/25" },
    { quality: 3, label: "Okay",         key: "3", bg: "bg-accent-correct dark:bg-accent-correct-dark",     shadow: "shadow-amber-500/25" },
    { quality: 4, label: "Gut",          key: "4", bg: "bg-accent-easy dark:bg-accent-easy-dark",           shadow: "shadow-green-500/25" },
  ];
</script>

<div class="grid w-full max-w-2xl grid-cols-2 gap-2 sm:grid-cols-4 sm:gap-3">
  {#each buttons as btn, index}
    <button
      disabled={!enabled}
      onclick={(event) => {
        event.stopPropagation();
        onRate(btn.quality);
      }}
      class="score-button {enabled && animate ? 'score-button-enter' : ''} {enabled ? `${btn.bg} ${btn.shadow} text-white` : 'cursor-not-allowed bg-secondary/15 text-secondary/55 shadow-none'} {smooth ? 'transition-colors duration-200' : ''} flex min-w-0 flex-col items-center gap-0.5 px-3 py-3 font-semibold sm:px-5"
      style:animation-delay={enabled && animate ? `${index * 28}ms` : "0ms"}
      title={enabled ? `Taste ${btn.key}` : "Erst nach dem Aufdecken verfügbar"}
    >
      <span class="text-lg leading-none">{btn.key}</span>
      <span class="text-xs opacity-90">{btn.label}</span>
    </button>
  {/each}
</div>

<style>
  .score-button-enter {
    animation: score-button-enter 210ms cubic-bezier(0.22, 1, 0.36, 1) both;
    will-change: transform;
  }

  @keyframes score-button-enter {
    0% {
      transform: translateY(6px) scale(0.975);
    }

    100% {
      transform: translateY(0) scale(1);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .score-button-enter {
      animation: none;
    }
  }
</style>
