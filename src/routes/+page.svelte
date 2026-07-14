<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { ArrowDown, ArrowUp, GripVertical, Search as SearchIcon, Settings as SettingsIcon, X } from "@lucide/svelte";
  import DeckList from "$lib/components/DeckList.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import TagList from "$lib/components/TagList.svelte";
  import ExamCalendar from "$lib/components/ExamCalendar.svelte";
  import DashboardExtras from "$lib/components/DashboardExtras.svelte";
  import SingleCardModule from "$lib/components/SingleCardModule.svelte";
  import TestView from "$lib/components/TestView.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { Card, DashboardStats, Deck } from "$lib/types";

  type DashboardModuleId = "brand" | "search" | "settings" | "focus" | "continue" | "timer" | "learning" | "problems" | "weekPlan" | "quickCapture" | "learningTime" | "milestones" | "tags" | "exams" | "decks";
  type SingleCardModuleId = `singleCard:${string}`;
  type DashboardModuleKey = DashboardModuleId | SingleCardModuleId;
  type ExtraDashboardModuleId = "continue" | "timer" | "problems" | "weekPlan" | "quickCapture" | "learningTime" | "milestones";

  const defaultModuleOrder: DashboardModuleId[] = ["brand", "search", "settings", "continue", "focus", "learning", "problems", "timer", "learningTime", "weekPlan", "quickCapture", "milestones", "tags", "exams", "decks"];
  const moduleTitles: Record<DashboardModuleId, string> = {
    brand: "Stapelweise",
    search: "Suche",
    settings: "Einstellungen",
    focus: "Kleine Runde",
    continue: "Weiterlernen",
    timer: "Lerntimer",
    learning: "Lernlage",
    problems: "Problemkarten",
    weekPlan: "Wochenplan",
    quickCapture: "Schnellerfassung",
    learningTime: "Lernzeit",
    milestones: "Meilensteine",
    tags: "Tags",
    exams: "Prüfungen & Ziele",
    decks: "Deine Stapel",
  };
  const dashboardLayoutStorageKey = "stapelweise.dashboard.modules.v5";
  const legacyDashboardLayoutStorageKey = "stapelweise.dashboard.modules.v4";
  const singleCardStorageKey = "stapelweise.dashboard.singleCards.v1";
  const compactModuleIds: DashboardModuleId[] = ["brand", "search", "settings"];

  const isSingleCardModule = (moduleId: string): moduleId is SingleCardModuleId => moduleId.startsWith("singleCard:");
  const isDashboardModuleKey = (moduleId: unknown): moduleId is DashboardModuleKey =>
    typeof moduleId === "string" && (defaultModuleOrder.includes(moduleId as DashboardModuleId) || isSingleCardModule(moduleId));
  const moduleTitle = (moduleId: DashboardModuleKey) => isSingleCardModule(moduleId) ? "Einzelkarte" : moduleTitles[moduleId];

  let view = $state<"decks" | "cards" | "study" | "search" | "settings" | "test">("decks");
  let activeDeck = $state<Deck | null>(null);
  let activeDeckIds = $state<string[]>([]);
  let activeTags = $state<string[]>([]);
  let activeCustomCards = $state<Card[]>([]);
  let activeDeckName = $state("");
  let activePracticeMode = $state(false);
  let dashboard = $state<DashboardStats | null>(null);
  let moduleOrder = $state<DashboardModuleKey[]>([...defaultModuleOrder]);
  let arrangingModules = $state(false);
  let showModulePicker = $state(false);
  let draggedModule = $state<DashboardModuleKey | null>(null);
  let dragTargetModule = $state<DashboardModuleKey | null>(null);
  let dragPlacement = $state<"before" | "after">("before");
  let activeSearchQuery = $state("");
  let singleCardSelections = $state<Record<string, string>>({});
  let dashboardCards = $state<Card[]>([]);
  let dashboardCardDeckSignature = $state("");
  let dashboardRevision = $state(0);
  let studyStartedAt = $state<number | null>(null);
  let studyReviews = $state(0);
  const weekDays = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

  onMount(() => {
    try {
      singleCardSelections = JSON.parse(localStorage.getItem(singleCardStorageKey) ?? "{}");
      const savedLayout = localStorage.getItem(dashboardLayoutStorageKey);
      if (savedLayout !== null) {
        const savedOrder = JSON.parse(savedLayout);
        const filteredOrder = Array.isArray(savedOrder)
          ? [...new Set(savedOrder.filter(isDashboardModuleKey))]
          : [...defaultModuleOrder];
        moduleOrder = filteredOrder.includes("settings") ? filteredOrder : ["settings", ...filteredOrder];
        return;
      }

      const legacySavedLayout = localStorage.getItem(legacyDashboardLayoutStorageKey);
      if (legacySavedLayout === null) {
        moduleOrder = [...defaultModuleOrder];
        localStorage.setItem(dashboardLayoutStorageKey, JSON.stringify(moduleOrder));
        return;
      }

      const legacyOrder = JSON.parse(legacySavedLayout);
      const knownLegacyModules = Array.isArray(legacyOrder)
        ? legacyOrder.filter((id: unknown): id is DashboardModuleId =>
            typeof id === "string" && defaultModuleOrder.includes(id as DashboardModuleId)
          )
        : [];
      const visibleCompactModules = compactModuleIds.filter((moduleId) =>
        moduleId === "settings" || knownLegacyModules.includes(moduleId)
      );
      const remainingLegacyModules = knownLegacyModules.filter((moduleId) => !compactModuleIds.includes(moduleId));
      moduleOrder = [...visibleCompactModules, ...remainingLegacyModules];
      localStorage.setItem(dashboardLayoutStorageKey, JSON.stringify(moduleOrder));
    } catch {
      moduleOrder = [...defaultModuleOrder];
    }
  });

  function saveModuleOrder(order: DashboardModuleKey[]) {
    const safeOrder: DashboardModuleKey[] = order.includes("settings") ? order : ["settings", ...order];
    moduleOrder = safeOrder;
    localStorage.setItem(dashboardLayoutStorageKey, JSON.stringify(safeOrder));
  }

  function moveModule(source: DashboardModuleKey, target: DashboardModuleKey, placement?: "before" | "after") {
    if (source === target) return;
    const sourceIndex = moduleOrder.indexOf(source);
    const targetIndexBeforeMove = moduleOrder.indexOf(target);
    const next = moduleOrder.filter((id) => id !== source);
    const targetIndex = next.indexOf(target);
    const insertAfter = placement ? placement === "after" : sourceIndex < targetIndexBeforeMove;
    const insertIndex = insertAfter ? targetIndex + 1 : targetIndex;
    next.splice(insertIndex, 0, source);
    saveModuleOrder(next);
  }

  function moveModuleBy(moduleId: DashboardModuleKey, offset: number) {
    const currentIndex = moduleOrder.indexOf(moduleId);
    const nextIndex = Math.max(0, Math.min(moduleOrder.length - 1, currentIndex + offset));
    if (currentIndex === nextIndex) return;
    const next = [...moduleOrder];
    next.splice(currentIndex, 1);
    next.splice(nextIndex, 0, moduleId);
    saveModuleOrder(next);
  }

  function handleModulePointerDown(event: PointerEvent, moduleId: DashboardModuleKey) {
    if (!arrangingModules || event.button !== 0) return;
    if ((event.target as HTMLElement).closest(".dashboard-order-button")) return;
    if (event.pointerType === "touch" && !(event.target as HTMLElement).closest(".dashboard-drag-handle")) return;
    draggedModule = moduleId;
    dragTargetModule = null;
    dragPlacement = "before";
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function handleModulePointerMove(event: PointerEvent) {
    if (!draggedModule) return;
    event.preventDefault();
    const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>("[data-dashboard-module]");
    const targetId = target?.dataset.dashboardModule as DashboardModuleKey | undefined;
    const nextTarget = targetId && targetId !== draggedModule ? targetId : null;
    if (dragTargetModule !== nextTarget) dragTargetModule = nextTarget;
    if (target && nextTarget) {
      const rect = target.getBoundingClientRect();
      dragPlacement = target.dataset.dashboardWidth === "full"
        ? event.clientY < rect.top + rect.height / 2 ? "before" : "after"
        : event.clientX < rect.left + rect.width / 2 ? "before" : "after";
    }
  }

  function finishModuleDrag(commit = true) {
    if (commit && draggedModule && dragTargetModule) moveModule(draggedModule, dragTargetModule, dragPlacement);
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
  }

  function resetModuleOrder() {
    saveModuleOrder([...defaultModuleOrder]);
    singleCardSelections = {};
    localStorage.removeItem(singleCardStorageKey);
    showModulePicker = false;
  }

  function removeModule(moduleId: DashboardModuleKey) {
    if (moduleId === "settings") return;
    saveModuleOrder(moduleOrder.filter((id) => id !== moduleId));
    if (isSingleCardModule(moduleId)) {
      const { [moduleId]: _, ...remainingSelections } = singleCardSelections;
      singleCardSelections = remainingSelections;
      localStorage.setItem(singleCardStorageKey, JSON.stringify(singleCardSelections));
    }
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
  }

  function addModule(moduleId: DashboardModuleId) {
    if (moduleOrder.includes(moduleId)) return;
    saveModuleOrder([...moduleOrder, moduleId]);
  }

  function addSingleCardModule() {
    const uniqueId = typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    saveModuleOrder([...moduleOrder, `singleCard:${uniqueId}`]);
  }

  function selectSingleCard(moduleId: SingleCardModuleId, cardId: string) {
    singleCardSelections = { ...singleCardSelections, [moduleId]: cardId };
    localStorage.setItem(singleCardStorageKey, JSON.stringify(singleCardSelections));
  }

  let hasDecks = $derived(deckStore.decks.length > 0);
  let hiddenModules = $derived(defaultModuleOrder.filter((moduleId) => !moduleOrder.includes(moduleId)));
  let primaryActionLabel = $derived.by(() => {
    if (!hasDecks) return "Ersten Stapel anlegen";
    if (dashboard && dashboard.total_cards === 0) return "Karten anlegen";
    if (dashboard && dashboard.due_cards > 0) return "Lernsitzung starten";
    return "Frei lernen";
  });
  let learningLoad = $derived.by(() => {
    if (!dashboard) return "Bereit";
    if (!hasDecks) return "Startklar";
    if (dashboard.due_cards === 0) return "Alles im Rhythmus";
    if (dashboard.due_cards <= 12) return "Kleine Runde";
    if (dashboard.due_cards <= 35) return "Guter Fokusblock";
    return "Aufholsession";
  });
  deckStore.load();

  $effect(() => {
    api.getDashboardStats().then((s) => (dashboard = s)).catch(() => {});
  });

  $effect(() => {
    if (!moduleOrder.some(isSingleCardModule)) return;
    const signature = deckStore.decks.map((deck: Deck) => deck.id).join(",");
    if (!signature || dashboardCardDeckSignature === signature) return;
    dashboardCardDeckSignature = signature;
    Promise.all(deckStore.decks.map((deck: Deck) => api.listCards(deck.id)))
      .then((cards) => (dashboardCards = cards.flat()))
      .catch(() => (dashboardCards = []));
  });

  function handleOpenSettings() {
    view = "settings";
  }

  function handleOpenSearch() {
    activeSearchQuery = "";
    view = "search";
  }

  $effect(() => {
    window.addEventListener("open-settings", handleOpenSettings);
    window.addEventListener("open-search", handleOpenSearch);
    return () => {
      window.removeEventListener("open-settings", handleOpenSettings);
      window.removeEventListener("open-search", handleOpenSearch);
    };
  });

  function handleSelectDeck(deck: Deck) {
    activeDeck = deck;
    view = "cards";
  }

  function handleStudyDeck(deck: Deck) {
    localStorage.setItem("stapelweise.learning.lastDeck.v1", deck.id);
    beginStudyTracking();
    activeDeck = deck;
    activeDeckIds = [deck.id];
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = deck.name;
    view = "study";
  }

  function handleStudyDecks(decks: Deck[]) {
    if (decks.length === 1) localStorage.setItem("stapelweise.learning.lastDeck.v1", decks[0].id);
    beginStudyTracking();
    activeDeckIds = decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} Stapel`;
    view = "study";
  }

  function handleStudyToday() {
    if (!dashboard || dashboard.due_cards === 0 || deckStore.decks.length === 0) return;
    beginStudyTracking();
    activeDeckIds = deckStore.decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = "Heute lernen";
    view = "study";
  }

  function handlePracticeDeck(deck: Deck) {
    localStorage.setItem("stapelweise.learning.lastDeck.v1", deck.id);
    beginStudyTracking();
    activeDeck = deck;
    activeDeckIds = [deck.id];
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = true;
    activeDeckName = `${deck.name} · Freie Übung`;
    view = "study";
  }

  function handlePracticeAllDecks() {
    if (deckStore.decks.length === 0) return;
    beginStudyTracking();
    activeDeckIds = deckStore.decks.map((deck) => deck.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = true;
    activeDeckName = "Freie Übung";
    view = "study";
  }

  function requestNewDeck() {
    window.dispatchEvent(new CustomEvent("stapelweise:new-deck"));
  }

  function requestNewExam() {
    window.dispatchEvent(new CustomEvent("stapelweise:new-exam"));
  }

  function handlePrimaryAction() {
    if (!hasDecks) {
      requestNewDeck();
      return;
    }
    if (dashboard && dashboard.total_cards === 0) {
      handleSelectDeck(deckStore.decks[0]);
      return;
    }
    if (dashboard && dashboard.due_cards > 0) {
      handleStudyToday();
      return;
    }
    handlePracticeAllDecks();
  }

  function handleStudyTags(tags: string[]) {
    beginStudyTracking();
    activeDeckIds = deckStore.decks.map((deck) => deck.id);
    activeTags = tags;
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = tags.length === 1 ? `#${tags[0]}` : `${tags.length} Tags`;
    view = "study";
  }

  function handleStudyExam(deckIds: string[], examName: string) {
    beginStudyTracking();
    activeDeckIds = deckIds;
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = `Prüfung: ${examName}`;
    view = "study";
  }

  function handleSimulateExam(deckIds: string[], examName: string) {
    activeDeckIds = deckIds;
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = `Simulation: ${examName}`;
    view = "test";
  }

  function goHome() {
    finishStudyTracking();
    refreshDashboard();
    view = "decks";
    activeDeck = null;
    activeDeckIds = [];
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = "";
  }

  function beginStudyTracking() {
    studyStartedAt = Date.now();
    studyReviews = 0;
  }

  function finishStudyTracking() {
    if (view !== "study" || !studyStartedAt) return;
    const seconds = Math.max(1, Math.min(8 * 60 * 60, Math.round((Date.now() - studyStartedAt) / 1000)));
    const now = new Date();
    const key = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const storageKey = "stapelweise.learning.activity.v1";
    try {
      const log = JSON.parse(localStorage.getItem(storageKey) ?? "{}") as Record<string, { reviews: number; seconds: number }>;
      const current = log[key] ?? { reviews: 0, seconds: 0 };
      log[key] = { reviews: current.reviews + studyReviews, seconds: current.seconds + seconds };
      localStorage.setItem(storageKey, JSON.stringify(log));
    } catch {}
    studyStartedAt = null;
    studyReviews = 0;
  }

  function handleProblemCards(cards: Card[], name: string) {
    if (cards.length === 0) return;
    beginStudyTracking();
    activeDeckIds = [];
    activeTags = [];
    activeCustomCards = cards;
    activePracticeMode = false;
    activeDeckName = name;
    view = "study";
  }

  function refreshDashboard() {
    dashboardRevision += 1;
    api.getDashboardStats().then((stats) => (dashboard = stats)).catch(() => {});
    dashboardCardDeckSignature = "";
  }
</script>

<div class="grid h-full w-full">
  {#if view === "cards" && activeDeck}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <CardEditor
        deck={activeDeck}
        onClose={goHome}
        onStudy={() => handleStudyDeck(activeDeck!)}
        onPractice={() => handlePracticeDeck(activeDeck!)}
      />
    </div>
  {:else if view === "study" && (activeDeckIds.length > 0 || activeTags.length > 0 || activeCustomCards.length > 0)}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <StudyView
        deckIds={activeDeckIds}
        tags={activeTags}
        customCards={activeCustomCards}
        deckName={activeDeckName}
        practiceMode={activePracticeMode}
        onReview={() => (studyReviews += 1)}
        onClose={goHome}
      />
    </div>
  {:else if view === "search"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <SearchView
        initialQuery={activeSearchQuery}
        onClose={goHome}
        onSelectCard={(deckId: string) => {
          const deck = deckStore.decks.find((d) => d.id === deckId);
          if (deck) {
            activeDeck = deck;
            view = "cards";
          }
        }}
      />
    </div>
  {:else if view === "settings"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <SettingsPanel onClose={goHome} />
    </div>
  {:else if view === "decks"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <div class="app-container py-6 sm:py-8">
        <div class="mb-5 flex min-h-9 flex-wrap items-center justify-end gap-2">
          {#if arrangingModules}
            <button onclick={resetModuleOrder} class="secondary-action px-3 py-1.5 text-xs">Zurücksetzen</button>
            <button
              onclick={() => (showModulePicker = !showModulePicker)}
              class="secondary-action px-3 py-1.5 text-xs"
              aria-expanded={showModulePicker}
            >Module hinzufügen{hiddenModules.length > 0 ? ` (${hiddenModules.length})` : ""}</button>
          {/if}
          <button
            onclick={() => {
              arrangingModules = !arrangingModules;
              draggedModule = null;
              dragTargetModule = null;
              dragPlacement = "before";
              if (!arrangingModules) showModulePicker = false;
            }}
            class="{arrangingModules ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs"
          >
            {arrangingModules ? "Fertig" : "Dashboard anordnen"}
          </button>
        </div>

        {#if arrangingModules && showModulePicker}
          <div class="mb-5 border-y border-[#D8DEE8] py-4 dark:border-[#303744]">
            <div class="flex flex-wrap items-center gap-2">
              <span class="mr-2 text-xs font-semibold uppercase text-secondary">Ausgeblendete Module</span>
              {#each hiddenModules as hiddenModule}
                <button onclick={() => addModule(hiddenModule)} class="secondary-action px-3 py-1.5 text-xs">
                  + {moduleTitles[hiddenModule]}
                </button>
              {/each}
              <button onclick={addSingleCardModule} class="secondary-action px-3 py-1.5 text-xs">+ Einzelkarte</button>
            </div>
          </div>
        {/if}

        <div class="dashboard-grid" role="list" aria-label="Dashboard-Module">
          {#each moduleOrder as moduleId (moduleId)}
            <section
              role="listitem"
              data-dashboard-module={moduleId}
              data-dashboard-width={moduleId === "exams" || moduleId === "decks" ? "full" : moduleId === "search" || moduleId === "settings" ? "icon" : "half"}
              class="dashboard-module {moduleId === 'search' || moduleId === 'settings' ? 'dashboard-icon-module col-span-1 lg:col-span-2' : moduleId === 'brand' ? 'col-span-2 lg:col-span-8' : moduleId === 'exams' || moduleId === 'decks' ? 'col-span-2 lg:col-span-12' : 'col-span-2 lg:col-span-6'} {arrangingModules ? 'dashboard-module-arranging' : ''} {draggedModule === moduleId ? 'dashboard-module-dragging' : ''} {dragTargetModule === moduleId ? `dashboard-module-drop-target dashboard-module-drop-${dragPlacement}` : ''}"
              onpointerdown={(event) => handleModulePointerDown(event, moduleId)}
              onpointermove={handleModulePointerMove}
              onpointerup={() => finishModuleDrag(true)}
              onpointercancel={() => finishModuleDrag(false)}
            >
              {#if arrangingModules}
                <div class="dashboard-module-toolbar">
                  <button
                    class="dashboard-drag-handle"
                    title="{moduleTitle(moduleId)} verschieben"
                    aria-label="{moduleTitle(moduleId)} verschieben"
                  ><GripVertical size={18} /></button>
                  <span class="min-w-0 flex-1 truncate text-xs font-semibold text-primary dark:text-primary-dark">{moduleTitle(moduleId)}</span>
                  <button
                    class="dashboard-order-button"
                    onclick={() => moveModuleBy(moduleId, -1)}
                    disabled={moduleOrder[0] === moduleId}
                    title="Nach oben"
                    aria-label="{moduleTitle(moduleId)} nach oben verschieben"
                  ><ArrowUp size={16} /></button>
                  <button
                    class="dashboard-order-button"
                    onclick={() => moveModuleBy(moduleId, 1)}
                    disabled={moduleOrder[moduleOrder.length - 1] === moduleId}
                    title="Nach unten"
                    aria-label="{moduleTitle(moduleId)} nach unten verschieben"
                  ><ArrowDown size={16} /></button>
                  {#if moduleId !== "settings"}
                    <button
                      class="dashboard-order-button dashboard-remove-button"
                      onclick={() => removeModule(moduleId)}
                      title="Entfernen"
                      aria-label="{moduleTitle(moduleId)} vom Dashboard entfernen"
                    ><X size={16} /></button>
                  {/if}
                </div>
              {/if}

              <div class:dashboard-module-content-disabled={arrangingModules}>
                {#if moduleId === "brand"}
                  <div class="surface-panel flex min-h-28 items-center gap-5 p-5 sm:p-6">
                    <div class="relative h-14 w-14 shrink-0" aria-hidden="true">
                      <span class="absolute left-1.5 top-1.5 h-12 w-12 bg-accent-correct"></span>
                      <span class="font-pixel absolute left-0 top-0 flex h-12 w-12 items-center justify-center border-[3px] border-[#111827] bg-white text-xl font-black text-[#111827] dark:border-[#F8FAFC] dark:bg-[#171B24] dark:text-[#F8FAFC]">S</span>
                      <span class="absolute left-1.5 top-1.5 h-2 w-2 bg-accent-hard"></span>
                      <span class="absolute bottom-3 right-3 h-2 w-2 bg-accent-correct"></span>
                    </div>
                    <p class="font-pixel min-w-0 text-xl font-bold text-primary dark:text-primary-dark sm:text-2xl">stapelweise</p>
                  </div>
                {:else if moduleId === "search"}
                  <button
                    onclick={handleOpenSearch}
                    class="surface-panel flex aspect-square w-full items-center justify-center text-primary transition-colors hover:border-accent-correct hover:text-accent-correct dark:text-primary-dark"
                    title="Karten durchsuchen"
                    aria-label="Karten durchsuchen"
                  ><SearchIcon size={34} strokeWidth={1.8} /></button>
                {:else if moduleId === "settings"}
                  <button
                    onclick={handleOpenSettings}
                    class="surface-panel flex aspect-square w-full items-center justify-center text-primary transition-colors hover:border-accent-correct hover:text-accent-correct dark:text-primary-dark"
                    title="Einstellungen öffnen"
                    aria-label="Einstellungen öffnen"
                  ><SettingsIcon size={34} strokeWidth={1.8} /></button>
                {:else if isSingleCardModule(moduleId)}
                  <SingleCardModule
                    decks={deckStore.decks}
                    cards={dashboardCards}
                    selectedCardId={singleCardSelections[moduleId] ?? ""}
                    onSelect={(cardId) => selectSingleCard(moduleId, cardId)}
                  />
                {:else if moduleId === "focus"}
                  {#if dashboard}
                    <div class="surface-panel h-full overflow-hidden">
                      <div class="p-5 sm:p-6">
                        <div class="flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between">
                          <div>
                            <p class="section-kicker mb-2">{learningLoad}</p>
                            <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">Heute lernen</h1>
                          </div>
                          <div class="rounded-lg border border-[#E4E7EC] bg-[#F8FAFC] px-4 py-3 text-left dark:border-[#2A303B] dark:bg-[#10131A] sm:text-right">
                            <p class="font-pixel text-2xl font-bold text-primary dark:text-primary-dark">{dashboard.due_cards}</p>
                            <p class="text-xs font-semibold uppercase tracking-wide text-secondary">Karten fällig</p>
                          </div>
                        </div>
                        <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
                          <button onclick={handlePrimaryAction} class="primary-action px-5 py-2.5 text-sm">{primaryActionLabel}</button>
                          <button onclick={requestNewExam} class="secondary-action px-5 py-2.5 text-sm">Prüfung planen</button>
                        </div>
                      </div>
                      <div class="grid grid-cols-3 border-t border-[#E4E7EC] bg-[#F8FAFC] dark:border-[#2A303B] dark:bg-[#10131A]">
                        <div class="border-r border-[#E4E7EC] p-4 dark:border-[#2A303B]">
                          <p class="text-xs font-medium text-secondary">Heute gelernt</p>
                          <p class="font-pixel mt-2 text-base font-bold text-accent-easy dark:text-accent-easy-dark">{dashboard.reviews_today}</p>
                        </div>
                        <div class="border-r border-[#E4E7EC] p-4 dark:border-[#2A303B]">
                          <p class="text-xs font-medium text-secondary">Serie</p>
                          <p class="font-pixel mt-2 text-base font-bold text-accent-hard dark:text-accent-hard-dark">{dashboard.streak_days} <span class="font-sans text-sm font-semibold text-secondary">Tage</span></p>
                        </div>
                        <div class="p-4">
                          <p class="text-xs font-medium text-secondary">Gesamt</p>
                          <p class="font-pixel mt-2 text-base font-bold text-primary dark:text-primary-dark">{dashboard.total_cards}</p>
                        </div>
                      </div>
                    </div>
                  {:else}
                    <div class="surface-panel h-full p-5 sm:p-6">
                      <p class="section-kicker mb-2">Kleine Runde</p>
                      <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">Heute lernen</h1>
                      <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
                        <button onclick={requestNewDeck} class="primary-action px-5 py-2.5 text-sm">Ersten Stapel anlegen</button>
                        <button onclick={requestNewExam} class="secondary-action px-5 py-2.5 text-sm">Prüfung planen</button>
                      </div>
                    </div>
                  {/if}
                {:else if moduleId === "learning"}
                  <aside class="surface-panel h-full p-5">
                    <p class="section-kicker mb-3">Lernlage</p>
                    {#if dashboard}
                      <div class="space-y-4">
                        <div>
                          <div class="mb-1 flex items-center justify-between text-sm">
                            <span class="font-semibold text-primary dark:text-primary-dark">Lernpensum</span>
                            <span class="text-secondary">{dashboard.due_cards === 0 ? "frei" : dashboard.due_cards <= 20 ? "normal" : "hoch"}</span>
                          </div>
                          <div class="h-2 overflow-hidden rounded-full bg-[#E4E7EC] dark:bg-[#2A303B]">
                            <div class="h-full rounded-full bg-accent-correct transition-all" style="width: {Math.min(100, dashboard.due_cards * 3)}%"></div>
                          </div>
                        </div>
                        <div>
                          <div class="mb-2 flex items-center justify-between text-sm">
                            <span class="font-semibold text-primary dark:text-primary-dark">Wochenrhythmus</span>
                            <span class="text-secondary">{dashboard.streak_days > 0 ? "aktiv" : "neu starten"}</span>
                          </div>
                          <div class="grid grid-cols-7 gap-1.5">
                            {#each weekDays as day, index}
                              {@const active = index >= Math.max(0, 7 - dashboard.streak_days)}
                              <div class="flex flex-col items-center gap-1">
                                <div class="h-2 w-full rounded-full {active ? 'bg-accent-easy dark:bg-accent-easy-dark' : 'bg-[#E4E7EC] dark:bg-[#2A303B]'}"></div>
                                <span class="text-[10px] font-semibold text-secondary">{day}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                        <div class="grid grid-cols-2 gap-3">
                          <div class="rounded-lg border border-[#E4E7EC] p-3 dark:border-[#2A303B]">
                            <p class="text-xs text-secondary">Ø Ease</p>
                            <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{dashboard.avg_ease_factor.toFixed(2)}</p>
                          </div>
                          <div class="rounded-lg border border-[#E4E7EC] p-3 dark:border-[#2A303B]">
                            <p class="text-xs text-secondary">Bibliothek</p>
                            <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{deckStore.decks.length}</p>
                          </div>
                        </div>
                      </div>
                    {:else}
                      <div class="grid grid-cols-3 gap-3">
                        <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
                        <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
                        <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
                      </div>
                    {/if}
                  </aside>
                {:else if moduleId === "continue" || moduleId === "timer" || moduleId === "problems" || moduleId === "weekPlan" || moduleId === "quickCapture" || moduleId === "learningTime" || moduleId === "milestones"}
                  <DashboardExtras
                    moduleId={moduleId as ExtraDashboardModuleId}
                    decks={deckStore.decks}
                    {dashboard}
                    refreshToken={dashboardRevision}
                    onStudyDeck={handleStudyDeck}
                    onStudyCards={handleProblemCards}
                    onStudyToday={handlePrimaryAction}
                    onCardCreated={refreshDashboard}
                  />
                {:else if moduleId === "tags"}
                  <TagList onStudyTags={handleStudyTags} />
                {:else if moduleId === "exams"}
                  <ExamCalendar onStudyExam={handleStudyExam} onSimulateExam={handleSimulateExam} />
                {:else if moduleId === "decks"}
                  <DeckList
                    onSelectDeck={handleSelectDeck}
                    onStudyDeck={handleStudyDeck}
                    onPracticeDeck={handlePracticeDeck}
                    onStudyDecks={handleStudyDecks}
                    onLibraryChanged={refreshDashboard}
                    refreshToken={dashboardRevision}
                  />
                {/if}
              </div>
            </section>
          {/each}
        </div>

        {#if moduleOrder.length === 0}
          <div class="flex min-h-72 flex-col items-center justify-center text-center">
            <p class="font-pixel text-lg text-primary dark:text-primary-dark">Dein Dashboard ist leer</p>
            <p class="mt-3 max-w-md text-sm text-secondary">Füge genau die Module hinzu, die du beim Lernen sehen möchtest.</p>
            <button
              onclick={() => {
                arrangingModules = true;
                showModulePicker = true;
              }}
              class="primary-action mt-5 px-4 py-2 text-sm"
            >Module hinzufügen</button>
          </div>
        {/if}
      </div>
    </div>
  {:else if view === "test"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <TestView
        deckIds={activeDeckIds}
        tags={activeTags}
        testName={activeDeckName}
        onClose={goHome}
        onStudyFailed={(cards) => {
          activeDeckIds = [];
          activeTags = [];
          activeCustomCards = cards;
          activePracticeMode = false;
          activeDeckName = `${cards.length} Falsche Testkarten`;
          view = "study";
        }}
      />
    </div>
  {/if}
</div>
