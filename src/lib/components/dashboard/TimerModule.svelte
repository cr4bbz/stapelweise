<!-- Study timer module: a self-contained Pomodoro-style countdown whose state
     survives reloads and view changes via localStorage. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";

  const timerStorageKey = "stapelweise.learning.timer.v1";

  let timerMinutes = $state(25);
  let timerRemaining = $state(25 * 60);
  let timerRunning = $state(false);
  let timerEndAt = $state<number | null>(null);

  function formatTimer(seconds: number) {
    const minutes = Math.floor(seconds / 60);
    return `${String(minutes).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
  }

  function persist() {
    localStorage.setItem(timerStorageKey, JSON.stringify({
      minutes: timerMinutes,
      remaining: timerRemaining,
      running: timerRunning,
      endAt: timerEndAt,
    }));
  }

  function selectTimer(minutes: number) {
    timerMinutes = minutes;
    timerRemaining = minutes * 60;
    timerRunning = false;
    timerEndAt = null;
    persist();
  }

  function toggleTimer() {
    if (!timerRunning && timerRemaining === 0) timerRemaining = timerMinutes * 60;
    timerRunning = !timerRunning;
    if (timerRunning) {
      timerEndAt = Date.now() + timerRemaining * 1000;
    } else if (timerEndAt) {
      timerRemaining = Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000));
      timerEndAt = null;
    }
    persist();
  }

  function resetTimer() {
    timerRunning = false;
    timerEndAt = null;
    timerRemaining = timerMinutes * 60;
    persist();
  }

  onMount(() => {
    try {
      const saved = JSON.parse(localStorage.getItem(timerStorageKey) ?? "null");
      if (saved) {
        timerMinutes = Number(saved.minutes) || 25;
        timerRunning = Boolean(saved.running);
        timerEndAt = typeof saved.endAt === "number" ? saved.endAt : null;
        timerRemaining = timerRunning && timerEndAt
          ? Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000))
          : Number(saved.remaining) || timerMinutes * 60;
        if (timerRemaining === 0) timerRunning = false;
      }
    } catch {
      // Ignore malformed persisted timer state.
    }

    const interval = setInterval(() => {
      if (!timerRunning || !timerEndAt) return;
      timerRemaining = Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000));
      if (timerRemaining === 0) {
        timerRunning = false;
        timerEndAt = null;
        persist();
      }
    }, 500);

    return () => clearInterval(interval);
  });
</script>

<div class="surface-panel h-full p-5">
  <div class="flex items-start justify-between gap-4">
    <p class="section-kicker mb-2">{t("Lerntimer")}</p>
    <span class="font-pixel text-2xl text-primary dark:text-primary-dark" aria-live="polite">{formatTimer(timerRemaining)}</span>
  </div>
  <div class="module-accent-track mt-4 h-2 overflow-hidden rounded-full">
    <div class="module-accent-fill h-full" style="width: {100 - (timerRemaining / (timerMinutes * 60)) * 100}%"></div>
  </div>
  <div class="mt-4 flex flex-wrap items-center justify-between gap-2">
    <div class="flex items-center gap-2">
      {#each [15, 25, 45] as minutes}
        <button onclick={() => selectTimer(minutes)} disabled={timerRunning} class="{timerMinutes === minutes ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs disabled:opacity-40">{minutes} {t("Min.")}</button>
      {/each}
    </div>
    <div class="flex items-center gap-2">
      <button onclick={resetTimer} class="secondary-action px-3 py-1.5 text-xs">{t("reset")}</button>
      <button onclick={toggleTimer} class="primary-action min-w-20 px-3 py-1.5 text-xs">{timerRunning ? t("Pause") : timerRemaining === 0 ? t("Neu starten") : t("Start")}</button>
    </div>
  </div>
</div>
