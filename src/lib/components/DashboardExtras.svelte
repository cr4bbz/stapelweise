<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import type { Card, CardState, DashboardStats, Deck } from "$lib/types";

  type ExtraModuleId = "continue" | "problems" | "weekPlan" | "quickCapture" | "learningTime" | "milestones" | "timer";
  type ActivityDay = { reviews: number; seconds: number };
  type ActivityLog = Record<string, ActivityDay>;

  let {
    moduleId,
    decks = [],
    dashboard = null,
    refreshToken = 0,
    onStudyDeck = (_deck: Deck) => {},
    onStudyCards = (_cards: Card[], _name: string) => {},
    onStudyToday = () => {},
    onCardCreated = () => {},
  } = $props<{
    moduleId: ExtraModuleId;
    decks?: Deck[];
    dashboard?: DashboardStats | null;
    refreshToken?: number;
    onStudyDeck?: (deck: Deck) => void;
    onStudyCards?: (cards: Card[], name: string) => void;
    onStudyToday?: () => void;
    onCardCreated?: () => void;
  }>();

  const activityStorageKey = "stapelweise.learning.activity.v1";
  const lastDeckStorageKey = "stapelweise.learning.lastDeck.v1";
  const timerStorageKey = "stapelweise.learning.timer.v1";

  let activity = $state<ActivityLog>({});
  let problemCards = $state<Card[]>([]);
  let problemLoading = $state(false);
  let totalReviews = $state(0);
  let selectedDeckId = $state("");
  let quickFront = $state("");
  let quickBack = $state("");
  let quickSaving = $state(false);
  let quickMessage = $state("");
  let timerMinutes = $state(25);
  let timerRemaining = $state(25 * 60);
  let timerRunning = $state(false);
  let timerEndAt = $state<number | null>(null);
  let loadedProblemDecks = "";
  let loadedMilestoneDecks = "";

  const dateKey = (date: Date) => {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  };

  const weekDates = $derived.by(() => {
    const today = new Date();
    const monday = new Date(today);
    const day = today.getDay() || 7;
    monday.setDate(today.getDate() - day + 1);
    const formatter = new Intl.DateTimeFormat(settingsStore.current.ui_language, { weekday: "short" });
    return Array.from({ length: 7 }, (_, index) => {
      const date = new Date(monday);
      date.setDate(monday.getDate() + index);
      const key = dateKey(date);
      const storedReviews = activity[key]?.reviews ?? 0;
      const reviews = key === dateKey(today) ? Math.max(storedReviews, dashboard?.reviews_today ?? 0) : storedReviews;
      return { name: formatter.format(date), key, reviews, seconds: activity[key]?.seconds ?? 0, today: key === dateKey(today) };
    });
  });

  let weekReviews = $derived(weekDates.reduce((sum, day) => sum + day.reviews, 0));
  let weekSeconds = $derived(weekDates.reduce((sum, day) => sum + day.seconds, 0));
  let todaySeconds = $derived(activity[dateKey(new Date())]?.seconds ?? 0);
  let lastDeck = $derived.by(() => {
    if (decks.length === 0) return null;
    const savedId = typeof localStorage === "undefined" ? "" : localStorage.getItem(lastDeckStorageKey) ?? "";
    return decks.find((deck: Deck) => deck.id === savedId) ?? decks[0];
  });

  let milestones = $derived.by(() => {
    const cards = dashboard?.total_cards ?? 0;
    const streak = dashboard?.streak_days ?? 0;
    return [
      { label: t("Erste Runde"), value: totalReviews, target: 10, suffix: t("Wiederholungen") },
      { label: t("Eigene Bibliothek"), value: cards, target: 50, suffix: t("Karten") },
      { label: t("Im Rhythmus"), value: streak, target: 7, suffix: t("Tage in Folge") },
      { label: t("Fest verankert"), value: totalReviews, target: 100, suffix: t("Wiederholungen") },
    ];
  });

  function readActivity() {
    try {
      activity = JSON.parse(localStorage.getItem(activityStorageKey) ?? "{}") as ActivityLog;
    } catch {
      activity = {};
    }
  }

  async function loadProblemCards() {
    if (decks.length === 0) return;
    problemLoading = true;
    try {
      const cards = (await Promise.all(decks.map((deck: Deck) => api.listCards(deck.id)))).flat();
      const states = await Promise.all(cards.map((card) => api.getCardState(card.id)));
      problemCards = cards
        .map((card, index) => ({ card, state: states[index] }))
        .filter((entry): entry is { card: Card; state: CardState } => Boolean(entry.state))
        .filter(({ state }) => state.total_reviews > 0 && (state.correct_streak < 2 || state.ease_factor < 2.3))
        .sort((a, b) => a.state.correct_streak - b.state.correct_streak || a.state.ease_factor - b.state.ease_factor)
        .slice(0, 20)
        .map(({ card }) => card);
    } finally {
      problemLoading = false;
    }
  }

  async function loadTotalReviews() {
    if (decks.length === 0) return;
    const stats = await Promise.all(decks.map((deck: Deck) => api.getDeckStats(deck.id)));
    totalReviews = stats.reduce((sum, item) => sum + item.total_reviews_sum, 0);
  }

  async function createQuickCard() {
    if (!selectedDeckId || !quickFront.trim() || !quickBack.trim()) return;
    quickSaving = true;
    quickMessage = "";
    try {
      await api.createCard(selectedDeckId, quickFront.trim(), quickBack.trim());
      quickFront = "";
      quickBack = "";
      quickMessage = t("Karte angelegt");
      onCardCreated();
    } catch {
      quickMessage = t("Karte konnte nicht angelegt werden");
    } finally {
      quickSaving = false;
    }
  }

  function formatDuration(seconds: number) {
    if (seconds < 60) return `< 1 ${t("Min.")}`;
    const minutes = Math.round(seconds / 60);
    if (minutes < 60) return `${minutes} ${t("Min.")}`;
    const hours = Math.floor(minutes / 60);
    const rest = minutes % 60;
    return rest > 0 ? `${hours} ${t("Std.")} ${rest} ${t("Min.")}` : `${hours} ${t("Std.")}`;
  }

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
    }));
  }

  function selectTimer(minutes: number) {
    timerMinutes = minutes;
    timerRemaining = minutes * 60;
    timerRunning = false;
    timerEndAt = null;
    persistTimer();
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
    persistTimer();
  }

  function resetTimer() {
    timerRunning = false;
    timerEndAt = null;
    timerRemaining = timerMinutes * 60;
    persistTimer();
  }

  $effect(() => {
    const deckSignature = `${decks.map((deck: Deck) => deck.id).join(",")}:${refreshToken}`;
    if (!selectedDeckId && decks[0]) selectedDeckId = decks[0].id;

    if (moduleId === "problems" && deckSignature && loadedProblemDecks !== deckSignature) {
      loadedProblemDecks = deckSignature;
      void loadProblemCards();
    }
    if (moduleId === "milestones" && deckSignature && loadedMilestoneDecks !== deckSignature) {
      loadedMilestoneDecks = deckSignature;
      void loadTotalReviews();
    }
  });

  onMount(() => {
    readActivity();

    let interval: ReturnType<typeof setInterval> | undefined;
    if (moduleId === "timer") {
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
      } catch {}

      interval = setInterval(() => {
        if (!timerRunning || !timerEndAt) return;
        timerRemaining = Math.max(0, Math.ceil((timerEndAt - Date.now()) / 1000));
        if (timerRemaining === 0) {
          timerRunning = false;
          timerEndAt = null;
          persistTimer();
        }
      }, 500);
    }

    return () => interval && clearInterval(interval);
  });
</script>

{#if moduleId === "continue"}
  <div class="surface-panel h-full p-5">
    <p class="section-kicker mb-3">{t("Weiterlernen")}</p>
    {#if lastDeck}
      <h2 class="font-card-serif text-xl font-normal text-primary dark:text-primary-dark">{lastDeck.name}</h2>
      <button onclick={() => onStudyDeck(lastDeck!)} class="primary-action mt-4 px-4 py-2 text-sm">{t("Runde fortsetzen")}</button>
    {:else}
      <p class="text-sm text-secondary">{t("Sobald du einen Stapel angelegt hast, kannst du hier direkt weitermachen.")}</p>
    {/if}
  </div>
{:else if moduleId === "problems"}
  <div class="surface-panel h-full p-5">
    <div class="flex items-start justify-between gap-4">
      <p class="section-kicker mb-2">{t("Problemkarten")}</p>
      <span class="font-pixel text-lg text-primary dark:text-primary-dark">{problemLoading ? "..." : problemCards.length}</span>
    </div>
    <button
      onclick={() => onStudyCards(problemCards, t("Problemkarten"))}
      disabled={problemLoading || problemCards.length === 0}
      class="primary-action mt-5 px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-40"
    >{t("Gezielt üben")}</button>
  </div>
{:else if moduleId === "weekPlan"}
  <div class="surface-panel h-full p-5">
    <div class="flex items-start justify-between gap-4">
      <div>
        <p class="section-kicker mb-2">{t("Wochenplan")}</p>
        <p class="text-sm text-secondary">{weekReviews} {t("Wiederholungen")} {t("in dieser Woche")}</p>
      </div>
      <button onclick={onStudyToday} disabled={!dashboard || dashboard.due_cards === 0} class="secondary-action px-3 py-1.5 text-xs disabled:opacity-40">{t("todayLearn")}</button>
    </div>
    <div class="mt-5 grid grid-cols-7 gap-2">
      {#each weekDates as day}
        <div class="flex min-w-0 flex-col items-center gap-2">
          <div class="module-accent-track flex h-16 w-full items-end overflow-hidden rounded">
            <div class="module-accent-fill w-full" style="height: {Math.max(day.reviews > 0 ? 14 : 4, Math.min(100, day.reviews * 7))}%"></div>
          </div>
          <span class="text-[10px] font-semibold {day.today ? 'module-accent-text' : 'text-secondary'}">{day.name}</span>
          <span class="text-xs text-primary dark:text-primary-dark">{day.reviews}</span>
        </div>
      {/each}
    </div>
  </div>
{:else if moduleId === "quickCapture"}
  <form class="surface-panel h-full p-5" onsubmit={(event) => { event.preventDefault(); void createQuickCard(); }}>
    <div class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
      <p class="section-kicker mb-2">{t("Schnellerfassung")}</p>
      <select bind:value={selectedDeckId} class="module-accent-input rounded-md px-3 py-2 text-sm outline-none">
        {#each decks as deck}<option value={deck.id}>{deck.name}</option>{/each}
      </select>
    </div>
    <div class="mt-4 grid gap-3 sm:grid-cols-2">
      <input bind:value={quickFront} aria-label={t("Vorderseite")} placeholder={t("Vorderseite")} class="module-accent-input rounded-md px-3 py-2.5 text-sm outline-none" />
      <input bind:value={quickBack} aria-label={t("Rückseite")} placeholder={t("Rückseite")} class="module-accent-input rounded-md px-3 py-2.5 text-sm outline-none" />
    </div>
    <div class="mt-3 flex items-center justify-between gap-3">
      <span class="text-xs text-secondary" aria-live="polite">{quickMessage}</span>
      <button type="submit" disabled={quickSaving || !selectedDeckId || !quickFront.trim() || !quickBack.trim()} class="primary-action px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-40">{quickSaving ? t("Speichert...") : t("Karte anlegen")}</button>
    </div>
  </form>
{:else if moduleId === "learningTime"}
  <div class="surface-panel h-full p-5">
    <p class="section-kicker mb-3">{t("Lernzeit")}</p>
    <div class="grid grid-cols-2 gap-3">
      <div class="module-accent-subpanel rounded-lg p-3">
        <p class="text-xs text-secondary">{t("Heute")}</p>
        <p class="font-pixel mt-2 text-sm text-primary dark:text-primary-dark">{formatDuration(todaySeconds)}</p>
      </div>
      <div class="module-accent-subpanel rounded-lg p-3">
        <p class="text-xs text-secondary">{t("Diese Woche")}</p>
        <p class="font-pixel mt-2 text-sm text-primary dark:text-primary-dark">{formatDuration(weekSeconds)}</p>
      </div>
    </div>
  </div>
{:else if moduleId === "milestones"}
  <div class="surface-panel h-full p-5">
    <div class="flex items-start justify-between gap-4">
      <p class="section-kicker mb-2">{t("Meilensteine")}</p>
      <span class="module-accent-text font-pixel text-lg">{milestones.filter((item) => item.value >= item.target).length}/{milestones.length}</span>
    </div>
    <div class="mt-5 grid grid-cols-2 gap-3">
      {#each milestones as item}
        {@const complete = item.value >= item.target}
        <div class="module-accent-subpanel rounded-lg p-3">
          <div class="flex items-center justify-between gap-2">
            <p class="text-sm font-semibold text-primary dark:text-primary-dark">{item.label}</p>
            <span class="text-xs font-semibold {complete ? 'module-accent-text' : 'text-secondary'}">{complete ? t("Erreicht") : `${Math.min(item.value, item.target)}/${item.target}`}</span>
          </div>
          <p class="mt-1 text-xs text-secondary">{item.suffix}</p>
          <div class="module-accent-track mt-3 h-1.5 overflow-hidden rounded-full">
            <div class="module-accent-fill h-full rounded-full" style="width: {Math.min(100, (item.value / item.target) * 100)}%"></div>
          </div>
        </div>
      {/each}
    </div>
  </div>
{:else if moduleId === "timer"}
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
{/if}
