<script lang="ts">
  import { t } from "$lib/i18n";
  let {
    enabled = false,
    animate = true,
    smooth = true,
    passThreshold = 3,
    onRate = (_quality: number) => {},
  } = $props();

  const buttons = [
    { quality: 1, label: "again", key: "1", bg: "bg-accent-incorrect dark:bg-accent-incorrect-dark" },
    { quality: 2, label: "hard",  key: "2", bg: "bg-accent-hard dark:bg-accent-hard-dark" },
    { quality: 3, label: "okay",  key: "3", bg: "bg-accent-correct dark:bg-accent-correct-dark" },
    { quality: 4, label: "good",  key: "4", bg: "bg-accent-easy dark:bg-accent-easy-dark" },
  ];
</script>

<div class="w-full max-w-2xl">
  <p class="mb-2 text-center text-xs text-secondary">
    {t("scorePassesFrom")} {t(buttons.find((button) => button.quality === passThreshold)?.label ?? "okay")}
  </p>
  <div class="grid grid-cols-2 gap-2 sm:grid-cols-4 sm:gap-3">
  {#each buttons as btn, index}
    <button
      disabled={!enabled}
      onclick={(event) => {
        event.stopPropagation();
        onRate(btn.quality);
      }}
      class="score-button {enabled && animate ? 'score-button-enter' : ''} {enabled ? `${btn.bg} text-white` : 'cursor-not-allowed border border-secondary/25 bg-secondary/20 text-secondary/85 shadow-none'} {smooth ? 'transition-colors duration-200' : ''} flex min-w-0 flex-col items-center gap-0.5 px-3 py-3 font-semibold sm:px-5"
      style:animation-delay={enabled && animate ? `${index * 28}ms` : "0ms"}
      aria-label={`${t(btn.label)} (${btn.quality >= passThreshold ? t("passed") : t("willRepeat")})`}
      title={enabled ? `Taste ${btn.key}` : "Erst nach dem Aufdecken verfügbar"}
    >
      <span class="text-lg leading-none">{btn.key}</span>
      <span class="text-xs opacity-90">{t(btn.label)}</span>
    </button>
  {/each}
  </div>
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
