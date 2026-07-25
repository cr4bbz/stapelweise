<script lang="ts">
  import { onMount } from "svelte";
  import { RotateCcw } from "@lucide/svelte";
  import { t } from "$lib/i18n";
  import { settingsStore } from "$lib/stores/settings.svelte";

  type TimerMode = "countdown" | "stopwatch";
  const timerStorageKey = "stapelweise.learning.timer.v1";
  const timerSliderSteps = 100;

  function readInitialTimerState() {
    if (typeof localStorage === "undefined") return null;
    try {
      const saved = JSON.parse(localStorage.getItem(timerStorageKey) ?? "null");
      return saved && typeof saved === "object" ? saved as Record<string, unknown> : null;
    } catch {
      return null;
    }
  }

  const initialTimerState = readInitialTimerState();
  const initialTimerMinutes = Math.max(1, Number(initialTimerState?.minutes) || 25);
  const initialTimerRunning = Boolean(initialTimerState?.running);
  const initialTimerEndAt = typeof initialTimerState?.endAt === "number" ? initialTimerState.endAt : null;
  const initialTimerRemaining = initialTimerRunning && initialTimerEndAt
    ? Math.max(0, Math.ceil((initialTimerEndAt - Date.now()) / 1000))
    : Math.max(0, Number(initialTimerState?.remaining) || initialTimerMinutes * 60);
  const initialStopwatchRunning = Boolean(initialTimerState?.stopwatchRunning);
  const initialStopwatchStartedAt = typeof initialTimerState?.stopwatchStartedAt === "number" ? initialTimerState.stopwatchStartedAt : null;
  const initialStopwatchElapsed = initialStopwatchRunning && initialStopwatchStartedAt
    ? Math.max(0, Math.floor((Date.now() - initialStopwatchStartedAt) / 1000))
    : Math.max(0, Number(initialTimerState?.stopwatchElapsed) || 0);

  let timerMinutes = $state(initialTimerMinutes);
  let timerRemaining = $state(initialTimerRemaining);
  let timerRunning = $state(initialTimerRunning);
  let timerEndAt = $state<number | null>(initialTimerEndAt);
  let timerMode = $state<TimerMode>(initialTimerState?.mode === "stopwatch" ? "stopwatch" : "countdown");
  let stopwatchElapsed = $state(initialStopwatchElapsed);
  let stopwatchRunning = $state(initialStopwatchRunning);
  let stopwatchStartedAt = $state<number | null>(initialStopwatchStartedAt);
  let timerProgress = $derived(timerMinutes > 0 ? Math.min(100, Math.max(0, 100 - (timerRemaining / (timerMinutes * 60)) * 100)) : 0);
  let minTimerMinutes = $derived(Math.min(475, Math.max(1, settingsStore.current.timer_min_minutes)));
  let maxTimerMinutes = $derived(Math.min(480, Math.max(minTimerMinutes + 5, settingsStore.current.timer_max_minutes)));
  let activeTimerRunning = $derived(timerMode === "countdown" ? timerRunning : stopwatchRunning);
  let displayedTimerSeconds = $derived(timerMode === "countdown" ? timerRemaining : stopwatchElapsed);
  let timerSliderPosition = $derived(timerSliderValue(timerMinutes));
  let timerRingProgress = $derived(timerMode === "countdown"
    ? timerProgress
    : ((stopwatchElapsed % 60) || (stopwatchElapsed > 0 ? 60 : 0)) / 60 * 100
  );

  function formatTimer(seconds: number) {
    const minutes = Math.floor(seconds / 60);
    return `${String(minutes).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
  }

  function persistTimer() {
    localStorage.setItem(timerStorageKey, JSON.stringify({
      minutes: timerMinutes,
      remaining: timerRemaining,
      running: timerRunning,
      endAt: timerEndAt,
      mode: timerMode,
      stopwatchElapsed,
      stopwatchRunning,
      stopwatchStartedAt,
    }));
  }

  function selectTimer(minutes: number) {
    timerMinutes = Math.min(maxTimerMinutes, Math.max(minTimerMinutes, minutes));
    timerRemaining = timerMinutes * 60;
    timerRunning = false;
    timerEndAt = null;
    persistTimer();
  }

  function toggleTimer() {
    if (timerMode === "stopwatch") {
      stopwatchRunning = !stopwatchRunning;
      if (stopwatchRunning) stopwatchStartedAt = Date.now() - stopwatchElapsed * 1000;
      else if (stopwatchStartedAt) {
        stopwatchElapsed = Math.max(0, Math.floor((Date.now() - stopwatchStartedAt) / 1000));
        stopwatchStartedAt = null;
      }
      persistTimer();
      return;
    }
    if (!timerRunning && timerRemaining === 0) timerRemaining = timerMinutes * 60;
    timerRunning = !timerRunning;
    if (timerRunning) timerEndAt = Date.now() + timerRemaining * 1000;
    else if (timerEndAt) {
      timerRemaining = Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000));
      timerEndAt = null;
    }
    persistTimer();
  }

  function resetTimer() {
    if (timerMode === "stopwatch") {
      stopwatchRunning = false;
      stopwatchStartedAt = null;
      stopwatchElapsed = 0;
    } else {
      timerRunning = false;
      timerEndAt = null;
      timerRemaining = timerMinutes * 60;
    }
    persistTimer();
  }

  function timerMinutesFromSlider(position: number) {
    const ratio = Math.min(1, Math.max(0, position / timerSliderSteps));
    const minutes = settingsStore.current.timer_slider_scale === "linear"
      ? minTimerMinutes + (maxTimerMinutes - minTimerMinutes) * ratio
      : minTimerMinutes * Math.pow(maxTimerMinutes / minTimerMinutes, ratio);
    return Math.min(maxTimerMinutes, Math.max(minTimerMinutes, Math.round(minutes / 5) * 5));
  }

  function timerSliderValue(minutes: number) {
    const boundedMinutes = Math.min(maxTimerMinutes, Math.max(minTimerMinutes, minutes));
    const ratio = settingsStore.current.timer_slider_scale === "linear"
      ? (boundedMinutes - minTimerMinutes) / (maxTimerMinutes - minTimerMinutes)
      : Math.log(boundedMinutes / minTimerMinutes) / Math.log(maxTimerMinutes / minTimerMinutes);
    return Math.round(ratio * timerSliderSteps);
  }

  function selectTimerMode(mode: TimerMode) {
    if (timerMode === mode) return;
    if (mode === "countdown" && timerRunning && timerEndAt) {
      timerRemaining = Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000));
      if (timerRemaining === 0) {
        timerRunning = false;
        timerEndAt = null;
      }
    }
    if (mode === "stopwatch" && stopwatchRunning && stopwatchStartedAt) {
      stopwatchElapsed = Math.max(0, Math.floor((Date.now() - stopwatchStartedAt) / 1000));
    }
    timerMode = mode;
    persistTimer();
  }

  $effect(() => {
    const clampedMinutes = Math.min(maxTimerMinutes, Math.max(minTimerMinutes, timerMinutes));
    if (clampedMinutes === timerMinutes) return;
    timerMinutes = clampedMinutes;
    if (timerRunning) {
      timerRemaining = Math.min(timerRemaining, timerMinutes * 60);
      timerEndAt = Date.now() + timerRemaining * 1000;
    } else timerRemaining = timerMinutes * 60;
  });

  onMount(() => {
    const interval = setInterval(() => {
      if (timerMode === "stopwatch") {
        if (stopwatchRunning && stopwatchStartedAt) stopwatchElapsed = Math.max(0, Math.floor((Date.now() - stopwatchStartedAt) / 1000));
      } else if (timerRunning && timerEndAt) {
        timerRemaining = Math.min(timerMinutes * 60, Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000)));
        if (timerRemaining === 0) {
          timerRunning = false;
          timerEndAt = null;
          persistTimer();
        }
      }
    }, 500);
    return () => clearInterval(interval);
  });
</script>

<div class="surface-panel flex h-full flex-col p-5">
  <div class="flex items-start justify-between gap-4">
    <p class="section-kicker">{timerMode === "stopwatch" ? t("Stoppuhr") : t("Lerntimer")}</p>
    <button onclick={resetTimer} class="icon-button shrink-0 !h-9 !w-9" title={t("reset")} aria-label={t("reset")}><RotateCcw size={17} /></button>
  </div>

  <div class="flex min-h-0 flex-1 items-center">
    <div class="relative mx-auto min-h-32 w-full max-w-sm">
      <button
        onclick={toggleTimer}
        class="absolute top-0 grid h-32 w-32 place-items-center rounded-full p-2.5 transition-[left,transform] duration-300 ease-out focus-visible:outline-none {timerMode === 'stopwatch' ? 'left-1/2 -translate-x-1/2' : 'left-1/4 -translate-x-1/2'}"
        aria-label={activeTimerRunning ? t("Pause") : timerMode === "countdown" && timerRemaining === 0 ? t("Neu starten") : t("Start")}
      >
        <span class="absolute inset-0 rounded-full" style={`background: conic-gradient(rgb(var(--color-accent-primary)) ${timerRingProgress}%, color-mix(in srgb, var(--color-surface) 78%, rgb(var(--color-accent-primary)) 22%) 0);`}></span>
        <span class="relative flex h-full w-full flex-col items-center justify-center overflow-hidden rounded-full bg-surface px-3 py-4 text-primary shadow-inner dark:text-primary-dark" style="container-type: inline-size">
          <span class="timer-time-display font-pixel whitespace-nowrap text-[clamp(0.75rem,16cqi,1.125rem)] leading-none" aria-live="polite">{formatTimer(displayedTimerSeconds)}</span>
          <span class="mt-2 whitespace-nowrap text-[9px] font-semibold uppercase leading-none tracking-[0.08em] text-secondary">{activeTimerRunning ? t("Pause") : t("Start")}</span>
        </span>
      </button>

      <div class="absolute left-1/2 top-0 flex h-32 w-1/2 min-w-0 items-center overflow-hidden px-4 transition-[opacity,transform] duration-200 {timerMode === 'countdown' ? 'translate-x-0 opacity-100' : 'pointer-events-none translate-x-2 opacity-0'}">
        <label class="relative block w-full">
          <span class="font-pixel absolute -top-6 left-1/2 -translate-x-1/2 whitespace-nowrap text-xs text-primary dark:text-primary-dark">{timerMinutes} {t("Min.")}</span>
          <input type="range" min="0" max={timerSliderSteps} step="1" value={timerSliderPosition} disabled={activeTimerRunning} oninput={(event) => selectTimer(timerMinutesFromSlider(Number(event.currentTarget.value)))} class="w-full accent-[rgb(var(--color-accent-primary))] disabled:cursor-not-allowed disabled:opacity-40" aria-label="Eigene Timer-Dauer in Minuten" />
          <span class="absolute top-4 flex w-full justify-between text-[10px] text-secondary"><span>{minTimerMinutes}</span><span>{maxTimerMinutes} {t("Min.")}</span></span>
        </label>
      </div>
    </div>
  </div>

  <footer class="mt-auto border-t border-current/10 pt-3">
    <div class="flex w-full overflow-hidden rounded-md border border-current/15" role="group" aria-label="Timer-Modus">
      <button class="min-w-0 flex-1 px-4 py-2 text-xs font-semibold transition-colors {timerMode === 'countdown' ? 'module-accent-fill text-white' : 'text-secondary hover:bg-current/5'}" onclick={() => selectTimerMode("countdown")} aria-pressed={timerMode === "countdown"}>{t("Lerntimer")}</button>
      <button class="min-w-0 flex-1 px-4 py-2 text-xs font-semibold transition-colors {timerMode === 'stopwatch' ? 'module-accent-fill text-white' : 'text-secondary hover:bg-current/5'}" onclick={() => selectTimerMode("stopwatch")} aria-pressed={timerMode === "stopwatch"}>{t("Stoppuhr")}</button>
    </div>
  </footer>
</div>
