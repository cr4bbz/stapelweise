<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { ArrowDown, ArrowUp, GripVertical, Plus, Search as SearchIcon, Settings as SettingsIcon, X } from "@lucide/svelte";
  import DeckModule from "$lib/components/DeckModule.svelte";
  import ArchiveModule from "$lib/components/ArchiveModule.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import TagList from "$lib/components/TagList.svelte";
  import ExamCalendar from "$lib/components/ExamCalendar.svelte";
  import ExamModule from "$lib/components/ExamModule.svelte";
  import DashboardExtras from "$lib/components/DashboardExtras.svelte";
  import SingleCardModule from "$lib/components/SingleCardModule.svelte";
  import TestView from "$lib/components/TestView.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { listenForDeepLinks, type StapelweiseDeepLink } from "$lib/deep-link";
  import type { Card, DashboardStats, Deck, Exam } from "$lib/types";
  import { t } from "$lib/i18n";

  type DashboardModuleId = "brand" | "search" | "settings" | "continue" | "timer" | "learning" | "problems" | "weekPlan" | "quickCapture" | "learningTime" | "milestones" | "tags" | "archive";
  type SingleCardModuleId = `singleCard:${string}`;
  type DeckModuleId = `deck:${string}`;
  type ExamModuleId = `exam:${string}`;
  type SpacerModuleId = `spacer:${string}`;
  type SpacerVariant = "blank" | "divider" | "note";
  type SpacerModuleConfig = { variant: SpacerVariant; note: string };
  type DashboardModuleKey = DashboardModuleId | SingleCardModuleId | DeckModuleId | ExamModuleId | SpacerModuleId;
  type ExtraDashboardModuleId = "continue" | "timer" | "problems" | "weekPlan" | "quickCapture" | "learningTime" | "milestones";
  type ModuleWidth = 2 | 3 | 4 | 5 | 6 | 8 | 12;
  type DeckModuleSize = 3 | 4 | 5;
  type ModuleTone = "primary" | "secondary" | "warm";
  type ModuleDragCandidate = { moduleId: DashboardModuleKey; startX: number; startY: number; pointerId: number };
  type ModuleDropAxis = "inline" | "block";

  const defaultModuleOrder: DashboardModuleId[] = ["brand", "search", "settings", "continue", "learning", "problems", "timer", "learningTime", "weekPlan", "quickCapture", "milestones", "tags"];
  const availableModuleIds: DashboardModuleId[] = [...defaultModuleOrder, "archive"];
  const moduleTitles: Record<DashboardModuleId, string> = {
    brand: "Stapelweise",
    search: "Suche",
    settings: "Einstellungen",
    continue: "Weiterlernen",
    timer: "Lerntimer",
    learning: "Lernlage",
    problems: "Problemkarten",
    weekPlan: "Wochenplan",
    quickCapture: "Schnellerfassung",
    learningTime: "Lernzeit",
    milestones: "Meilensteine",
    tags: "Tags",
    archive: "Archiv",
  };
  const dashboardLayoutStorageKey = "stapelweise.dashboard.modules.v5";
  const deckModuleSizeStorageKey = "stapelweise.dashboard.deckSizes.v1";
  const legacyDashboardLayoutStorageKey = "stapelweise.dashboard.modules.v4";
  const singleCardStorageKey = "stapelweise.dashboard.singleCards.v1";
  const spacerStorageKey = "stapelweise.dashboard.spacers.v1";
  const hiddenDeckModulesStorageKey = "stapelweise.dashboard.hiddenDecks.v1";
  const hiddenExamModulesStorageKey = "stapelweise.dashboard.hiddenExams.v1";
  const compactModuleIds: DashboardModuleId[] = ["brand", "search", "settings"];
  const deckModuleSizes: { width: DeckModuleSize; label: string }[] = [
    { width: 3, label: "Klein" },
    { width: 4, label: "Mittel" },
    { width: 5, label: "Groß" },
  ];
  const defaultModuleWidths: Record<DashboardModuleId, ModuleWidth> = {
    brand: 6,
    search: 2,
    settings: 2,
    continue: 4,
    timer: 4,
    learning: 6,
    problems: 4,
    weekPlan: 6,
    quickCapture: 6,
    learningTime: 4,
    milestones: 8,
    tags: 4,
    archive: 4,
  };

  const isSingleCardModule = (moduleId: string): moduleId is SingleCardModuleId => moduleId.startsWith("singleCard:");
  const isDeckModule = (moduleId: string): moduleId is DeckModuleId => moduleId.startsWith("deck:");
  const isExamModule = (moduleId: string): moduleId is ExamModuleId => moduleId.startsWith("exam:");
  const isSpacerModule = (moduleId: string): moduleId is SpacerModuleId => moduleId.startsWith("spacer:");
  const isDashboardModuleKey = (moduleId: unknown): moduleId is DashboardModuleKey =>
    typeof moduleId === "string" && (availableModuleIds.includes(moduleId as DashboardModuleId) || isSingleCardModule(moduleId) || isDeckModule(moduleId) || isExamModule(moduleId) || isSpacerModule(moduleId));
  const spacerTitle = (variant: SpacerVariant) => variant === "divider" ? "Trennlinie" : variant === "note" ? "Notizfläche" : "Leerraum";
  const moduleTitle = (moduleId: DashboardModuleKey) => isSingleCardModule(moduleId)
    ? "Einzelkarte"
    : isDeckModule(moduleId)
      ? deckStore.decks.find((deck) => deck.id === moduleId.slice("deck:".length))?.name ?? "Stapel"
    : isExamModule(moduleId)
      ? exams.find((exam) => exam.id === moduleId.slice("exam:".length))?.name ?? "Prüfung"
    : isSpacerModule(moduleId)
      ? spacerTitle(spacerModules[moduleId]?.variant ?? "blank")
      : t(moduleTitles[moduleId]);
  const moduleTone = (moduleId: DashboardModuleKey): ModuleTone => {
    if (isSingleCardModule(moduleId)) return "primary";
    if (isDeckModule(moduleId)) return "primary";
    if (isExamModule(moduleId)) return "warm";
    if (isSpacerModule(moduleId)) return "secondary";
    if (["settings", "continue", "learning", "weekPlan", "learningTime", "tags"].includes(moduleId)) return "secondary";
    if (["timer", "problems", "milestones"].includes(moduleId)) return "warm";
    return "primary";
  };

  let view = $state<"decks" | "cards" | "study" | "search" | "settings" | "test">("decks");
  let activeDeck = $state<Deck | null>(null);
  let activeDeckIds = $state<string[]>([]);
  let activeTags = $state<string[]>([]);
  let activeCustomCards = $state<Card[]>([]);
  let activeDeckName = $state("");
  let activePracticeMode = $state(false);
  let dashboard = $state<DashboardStats | null>(null);
  let moduleOrder = $state<DashboardModuleKey[]>([...defaultModuleOrder]);
  let deckModuleWidths = $state<Record<string, DeckModuleSize>>({});
  let arrangingModules = $state(false);
  let showModulePicker = $state(false);
  let draggedModule = $state<DashboardModuleKey | null>(null);
  let dragCandidate = $state<ModuleDragCandidate | null>(null);
  let dragTargetModule = $state<DashboardModuleKey | null>(null);
  let dragPlacement = $state<"before" | "after">("before");
  let dragAxis = $state<ModuleDropAxis>("inline");
  let activeSearchQuery = $state("");
  let singleCardSelections = $state<Record<string, string>>({});
  let spacerModules = $state<Record<string, SpacerModuleConfig>>({});
  let hiddenDeckIds = $state<string[]>([]);
  let hiddenExamIds = $state<string[]>([]);
  let exams = $state<Exam[]>([]);
  let examsLoaded = $state(false);
  let examRevision = $state(0);
  let showExamEditor = $state(false);
  let editingExam = $state<Exam | null>(null);
  let dashboardLayoutLoaded = $state(false);
  let showNewDeck = $state(false);
  let newDeckName = $state("");
  let newDeckError = $state<string | null>(null);
  let archiveConfirmDeck = $state<Deck | null>(null);
  let dashboardCards = $state<Card[]>([]);
  let dashboardCardDeckSignature = $state("");
  let dashboardRevision = $state(0);
  let studyStartedAt = $state<number | null>(null);
  let studyReviews = $state(0);
  let weekDays = $derived(["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"].map((day) => t(day)));

  function moduleWidth(moduleId: DashboardModuleKey): ModuleWidth {
    if (isDeckModule(moduleId)) return deckModuleWidths[moduleId] ?? 4;
    return isSingleCardModule(moduleId) || isExamModule(moduleId) || isSpacerModule(moduleId) ? 4 : defaultModuleWidths[moduleId];
  }

  function saveDeckModuleWidths(widths: Record<string, DeckModuleSize>) {
    deckModuleWidths = widths;
    localStorage.setItem(deckModuleSizeStorageKey, JSON.stringify(widths));
  }

  function setDeckModuleSize(moduleId: DeckModuleId, width: DeckModuleSize) {
    if (moduleWidth(moduleId) === width) return;
    const previousRects = new Map(
      Array.from(document.querySelectorAll<HTMLElement>("[data-dashboard-module]")).map((element) => [element, element.getBoundingClientRect()])
    );
    saveDeckModuleWidths({ ...deckModuleWidths, [moduleId]: width });
    void tick().then(() => {
      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
      for (const [element, previous] of previousRects) {
        const current = element.getBoundingClientRect();
        const deltaX = previous.left - current.left;
        const deltaY = previous.top - current.top;
        const scaleX = previous.width / current.width;
        const scaleY = previous.height / current.height;
        if (!deltaX && !deltaY && scaleX === 1 && scaleY === 1) continue;
        const resizedModule = element.dataset.dashboardModule === moduleId;
        if (resizedModule) element.style.zIndex = "20";
        const animation = element.animate(
          [
            { transformOrigin: "top left", transform: `translate(${deltaX}px, ${deltaY}px) scale(${scaleX}, ${scaleY})` },
            { transformOrigin: "top left", transform: "none" },
          ],
          { duration: 260, easing: "cubic-bezier(0.2, 0, 0, 1)" }
        );
        if (resizedModule) {
          void animation.finished.catch(() => {}).then(() => element.style.removeProperty("z-index"));
        }
      }
    });
  }

  onMount(() => {
    try {
      singleCardSelections = JSON.parse(localStorage.getItem(singleCardStorageKey) ?? "{}");
      const savedHiddenDeckIds = JSON.parse(localStorage.getItem(hiddenDeckModulesStorageKey) ?? "[]");
      hiddenDeckIds = Array.isArray(savedHiddenDeckIds)
        ? savedHiddenDeckIds.filter((deckId): deckId is string => typeof deckId === "string")
        : [];
      const savedHiddenExamIds = JSON.parse(localStorage.getItem(hiddenExamModulesStorageKey) ?? "[]");
      hiddenExamIds = Array.isArray(savedHiddenExamIds)
        ? savedHiddenExamIds.filter((examId): examId is string => typeof examId === "string")
        : [];
      const savedSpacers = JSON.parse(localStorage.getItem(spacerStorageKey) ?? "{}") as Record<string, unknown>;
      spacerModules = Object.fromEntries(
        Object.entries(savedSpacers).filter(([moduleId, config]) =>
          isSpacerModule(moduleId)
          && typeof config === "object"
          && config !== null
          && ["blank", "divider", "note"].includes((config as SpacerModuleConfig).variant)
        )
      ) as Record<string, SpacerModuleConfig>;
      localStorage.removeItem("stapelweise.dashboard.widths.v4");
      const savedDeckWidths = JSON.parse(localStorage.getItem(deckModuleSizeStorageKey) ?? "{}") as Record<string, unknown>;
      deckModuleWidths = Object.fromEntries(
        Object.entries(savedDeckWidths).filter(([moduleId, width]) =>
          isDeckModule(moduleId) && deckModuleSizes.some((size) => size.width === width)
        )
      ) as Record<string, DeckModuleSize>;
      localStorage.removeItem("stapelweise.dashboard.grid.v1");
      const savedLayout = localStorage.getItem(dashboardLayoutStorageKey);
      if (savedLayout !== null) {
        const savedOrder = JSON.parse(savedLayout);
        const filteredOrder = Array.isArray(savedOrder)
          ? [...new Set(savedOrder.filter(isDashboardModuleKey))]
          : [...defaultModuleOrder];
        moduleOrder = filteredOrder.includes("settings") ? filteredOrder : ["settings", ...filteredOrder];
      } else {
        const legacySavedLayout = localStorage.getItem(legacyDashboardLayoutStorageKey);
        if (legacySavedLayout === null) {
          moduleOrder = [...defaultModuleOrder];
          localStorage.setItem(dashboardLayoutStorageKey, JSON.stringify(moduleOrder));
        } else {
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
        }
      }
    } catch {
      moduleOrder = [...defaultModuleOrder];
    } finally {
      dashboardLayoutLoaded = true;
    }
  });

  onMount(() => {
    let unlisten = () => {};
    listenForDeepLinks(handleDeepLink).then((dispose) => (unlisten = dispose));
    return () => unlisten();
  });

  function saveModuleOrder(order: DashboardModuleKey[]) {
    const safeOrder: DashboardModuleKey[] = order.includes("settings") ? order : ["settings", ...order];
    moduleOrder = safeOrder;
    localStorage.setItem(dashboardLayoutStorageKey, JSON.stringify(safeOrder));
  }

  function moveModule(source: DashboardModuleKey, target: DashboardModuleKey, placement: "before" | "after") {
    if (source === target) return;
    const next = moduleOrder.filter((id) => id !== source);
    const targetIndex = next.indexOf(target);
    if (targetIndex < 0) return;
    next.splice(placement === "after" ? targetIndex + 1 : targetIndex, 0, source);
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
    dragCandidate = { moduleId, startX: event.clientX, startY: event.clientY, pointerId: event.pointerId };
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function handleModulePointerMove(event: PointerEvent, moduleId: DashboardModuleKey) {
    if (!dragCandidate || dragCandidate.moduleId !== moduleId || dragCandidate.pointerId !== event.pointerId) return;
    if (!draggedModule) {
      const distance = Math.hypot(event.clientX - dragCandidate.startX, event.clientY - dragCandidate.startY);
      if (distance < 6) return;
      draggedModule = moduleId;
    }

    event.preventDefault();
    const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>("[data-dashboard-module]");
    const targetId = target?.dataset.dashboardModule as DashboardModuleKey | undefined;
    dragTargetModule = targetId && targetId !== draggedModule ? targetId : null;
    if (!target || !dragTargetModule) return;

    const rect = target.getBoundingClientRect();
    const sourceWidth = moduleWidth(draggedModule);
    const targetWidth = moduleWidth(dragTargetModule);
    const columns = window.matchMedia("(min-width: 1024px)").matches ? 12 : window.matchMedia("(min-width: 640px)").matches ? 6 : 2;
    dragAxis = columns > 2 && sourceWidth + targetWidth <= columns ? "inline" : "block";
    dragPlacement = dragAxis === "inline"
      ? event.clientX < rect.left + rect.width / 2 ? "before" : "after"
      : event.clientY < rect.top + rect.height / 2 ? "before" : "after";
  }

  function finishModuleDrag(commit = true) {
    if (commit && draggedModule && dragTargetModule) moveModule(draggedModule, dragTargetModule, dragPlacement);
    dragCandidate = null;
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
    dragAxis = "inline";
  }

  function resetModuleOrder() {
    saveModuleOrder([...defaultModuleOrder]);
    deckModuleWidths = {};
    localStorage.removeItem(deckModuleSizeStorageKey);
    localStorage.removeItem("stapelweise.dashboard.grid.v1");
    singleCardSelections = {};
    localStorage.removeItem(singleCardStorageKey);
    spacerModules = {};
    localStorage.removeItem(spacerStorageKey);
    hiddenDeckIds = [];
    localStorage.removeItem(hiddenDeckModulesStorageKey);
    hiddenExamIds = [];
    localStorage.removeItem(hiddenExamModulesStorageKey);
    showModulePicker = false;
  }

  function removeModule(moduleId: DashboardModuleKey) {
    if (moduleId === "settings") return;
    saveModuleOrder(moduleOrder.filter((id) => id !== moduleId));
    if (isDeckModule(moduleId)) {
      const { [moduleId]: _width, ...remainingWidths } = deckModuleWidths;
      saveDeckModuleWidths(remainingWidths);
    }
    if (isSingleCardModule(moduleId)) {
      const { [moduleId]: _, ...remainingSelections } = singleCardSelections;
      singleCardSelections = remainingSelections;
      localStorage.setItem(singleCardStorageKey, JSON.stringify(singleCardSelections));
    }
    if (isDeckModule(moduleId)) {
      const deckId = moduleId.slice("deck:".length);
      hiddenDeckIds = [...new Set([...hiddenDeckIds, deckId])];
      localStorage.setItem(hiddenDeckModulesStorageKey, JSON.stringify(hiddenDeckIds));
    }
    if (isExamModule(moduleId)) {
      const examId = moduleId.slice("exam:".length);
      hiddenExamIds = [...new Set([...hiddenExamIds, examId])];
      localStorage.setItem(hiddenExamModulesStorageKey, JSON.stringify(hiddenExamIds));
    }
    if (isSpacerModule(moduleId)) {
      const { [moduleId]: _, ...remainingSpacers } = spacerModules;
      spacerModules = remainingSpacers;
      localStorage.setItem(spacerStorageKey, JSON.stringify(spacerModules));
    }
  }

  function addModule(moduleId: DashboardModuleId) {
    if (moduleOrder.includes(moduleId)) return;
    saveModuleOrder([...moduleOrder, moduleId]);
  }

  function addDeckModule(deckId: string) {
    const moduleId: DeckModuleId = `deck:${deckId}`;
    const wasHidden = hiddenDeckIds.includes(deckId);
    if (wasHidden) {
      hiddenDeckIds = hiddenDeckIds.filter((id) => id !== deckId);
      localStorage.setItem(hiddenDeckModulesStorageKey, JSON.stringify(hiddenDeckIds));
    }
    if (!moduleOrder.includes(moduleId)) saveModuleOrder([...moduleOrder, moduleId]);
  }

  function addExamModule(examId: string) {
    const moduleId: ExamModuleId = `exam:${examId}`;
    if (hiddenExamIds.includes(examId)) {
      hiddenExamIds = hiddenExamIds.filter((id) => id !== examId);
      localStorage.setItem(hiddenExamModulesStorageKey, JSON.stringify(hiddenExamIds));
    }
    if (!moduleOrder.includes(moduleId)) saveModuleOrder([...moduleOrder, moduleId]);
  }

  function addSingleCardModule() {
    const uniqueId = typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    saveModuleOrder([...moduleOrder, `singleCard:${uniqueId}`]);
  }

  function addSpacerModule(variant: SpacerVariant) {
    const uniqueId = typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    const moduleId: SpacerModuleId = `spacer:${uniqueId}`;
    spacerModules = { ...spacerModules, [moduleId]: { variant, note: "" } };
    localStorage.setItem(spacerStorageKey, JSON.stringify(spacerModules));
    saveModuleOrder([...moduleOrder, moduleId]);
  }

  function updateSpacerNote(moduleId: SpacerModuleId, note: string) {
    const config = spacerModules[moduleId] ?? { variant: "note" as const, note: "" };
    spacerModules = { ...spacerModules, [moduleId]: { ...config, note } };
    localStorage.setItem(spacerStorageKey, JSON.stringify(spacerModules));
  }

  function selectSingleCard(moduleId: SingleCardModuleId, cardId: string) {
    singleCardSelections = { ...singleCardSelections, [moduleId]: cardId };
    localStorage.setItem(singleCardStorageKey, JSON.stringify(singleCardSelections));
  }

  let hasDecks = $derived(deckStore.decks.length > 0);
  let hiddenModules = $derived(availableModuleIds.filter((moduleId) => !moduleOrder.includes(moduleId)));
  let hiddenDecks = $derived(deckStore.decks.filter((deck) => hiddenDeckIds.includes(deck.id)));
  let hiddenExams = $derived(exams.filter((exam) => hiddenExamIds.includes(exam.id)));
  deckStore.load();

  async function loadExams() {
    try {
      exams = await api.listExams();
      examsLoaded = true;
    } catch {
      // Keep the saved module layout intact until exams can be loaded again.
    } finally {
      examRevision += 1;
    }
  }

  onMount(() => {
    void loadExams();
  });

  $effect(() => {
    if (!dashboardLayoutLoaded) return;
    const activeDeckIds = new Set(deckStore.decks.map((deck) => deck.id));
    const activeExamIds = new Set(exams.map((exam) => exam.id));
    const withoutUnavailableModules = moduleOrder.filter((moduleId) =>
      (!isDeckModule(moduleId) || activeDeckIds.has(moduleId.slice("deck:".length)))
      && (!examsLoaded || !isExamModule(moduleId) || activeExamIds.has(moduleId.slice("exam:".length)))
    );
    const missingDeckModules = deckStore.decks
      .filter((deck) => !hiddenDeckIds.includes(deck.id) && !withoutUnavailableModules.includes(`deck:${deck.id}`))
      .map((deck) => `deck:${deck.id}` as DeckModuleId);
    const missingExamModules = (examsLoaded ? exams : [])
      .filter((exam) => !hiddenExamIds.includes(exam.id) && !withoutUnavailableModules.includes(`exam:${exam.id}`))
      .map((exam) => `exam:${exam.id}` as ExamModuleId);
    const nextOrder = [...withoutUnavailableModules, ...missingDeckModules, ...missingExamModules];
    if (nextOrder.length !== moduleOrder.length || nextOrder.some((moduleId, index) => moduleId !== moduleOrder[index])) {
      saveModuleOrder(nextOrder);
    }
  });

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
    activeDeck = decks.length === 1 ? decks[0] : null;
    activeDeckIds = decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} ${t("Stapel")}`;
    view = "study";
  }

  function handleStudyToday() {
    if (!dashboard || dashboard.due_cards === 0 || deckStore.decks.length === 0) return;
    beginStudyTracking();
    activeDeck = null;
    activeDeckIds = deckStore.decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = t("todayLearn");
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
    activeDeckName = `${deck.name} · ${t("freePractice")}`;
    view = "study";
  }

  function handleTestDeck(deck: Deck) {
    activeDeck = deck;
    activeDeckIds = [deck.id];
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = `${t("Stapelprüfung")}: ${deck.name}`;
    view = "test";
  }

  function handlePracticeAllDecks() {
    if (deckStore.decks.length === 0) return;
    beginStudyTracking();
    activeDeck = null;
    activeDeckIds = deckStore.decks.map((deck) => deck.id);
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = true;
    activeDeckName = t("freePractice");
    view = "study";
  }

  function requestNewDeck(name = "") {
    newDeckName = name;
    newDeckError = null;
    showNewDeck = true;
  }

  async function createDashboardDeck() {
    const name = newDeckName.trim();
    if (!name) return;
    newDeckError = null;
    try {
      await deckStore.create(name);
      newDeckName = "";
      showNewDeck = false;
      refreshDashboard();
    } catch {
      newDeckError = t("Fehler beim Erstellen des Stapels");
    }
  }

  async function archiveDeck(deck: Deck) {
    try {
      await deckStore.archive(deck.id);
      const moduleId: DeckModuleId = `deck:${deck.id}`;
      if (!moduleOrder.includes("archive")) saveModuleOrder([...moduleOrder.filter((id) => id !== moduleId), "archive"]);
      refreshDashboard();
    } finally {
      archiveConfirmDeck = null;
    }
  }

  async function restoreDeck(deck: Deck) {
    await deckStore.restore(deck.id);
    addDeckModule(deck.id);
    refreshDashboard();
  }

  async function archiveExam(exam: Exam) {
    await api.archiveExam(exam.id);
    if (!moduleOrder.includes("archive")) saveModuleOrder([...moduleOrder.filter((id) => id !== `exam:${exam.id}`), "archive"]);
    await loadExams();
  }

  async function restoreExam(exam: Exam) {
    await api.restoreExam(exam.id);
    await loadExams();
    addExamModule(exam.id);
  }

  async function handleDeepLink(link: StapelweiseDeepLink) {
    await deckStore.load();
    if (link.kind === "new-deck") {
      view = "decks";
      requestNewDeck(link.name ?? "");
      return;
    }
    const deck = deckStore.decks.find((candidate) => candidate.id === link.deck || candidate.name.toLowerCase() === link.deck.toLowerCase());
    if (!deck) return;
    activeDeck = deck;
    view = "cards";
    if (link.kind === "new-card") {
      await Promise.resolve();
      window.dispatchEvent(new CustomEvent("stapelweise:prefill-card", { detail: { deckId: deck.id, front: link.front, back: link.back } }));
    }
  }

  function requestNewExam() {
    editingExam = null;
    showExamEditor = true;
  }

  function requestEditExam(exam: Exam) {
    editingExam = exam;
    showExamEditor = true;
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
    activeDeck = null;
    activeDeckIds = deckStore.decks.map((deck) => deck.id);
    activeTags = tags;
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = tags.length === 1 ? `#${tags[0]}` : `${tags.length} ${t("Tags")}`;
    view = "study";
  }

  function handleStudyExam(deckIds: string[], examName: string) {
    beginStudyTracking();
    activeDeck = null;
    activeDeckIds = deckIds;
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = `${t("Prüfung")}: ${examName}`;
    view = "study";
  }

  function handleSimulateExam(deckIds: string[], examName: string) {
    activeDeck = null;
    activeDeckIds = deckIds;
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = `${t("Simulation")}: ${examName}`;
    view = "test";
  }

  function closeTest() {
    if (activeDeck) {
      activeDeckIds = [];
      activeTags = [];
      activeCustomCards = [];
      activePracticeMode = false;
      activeDeckName = "";
      view = "cards";
      return;
    }
    goHome();
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
    activeDeck = null;
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

  function closeStudy() {
    finishStudyTracking();
    refreshDashboard();
    if (activeDeck) {
      view = "cards";
      activeDeckIds = [];
      activeTags = [];
      activeCustomCards = [];
      activePracticeMode = false;
      activeDeckName = "";
      return;
    }
    goHome();
  }
</script>

<div class="grid h-full w-full">
  <ExamCalendar
    open={showExamEditor}
    exam={editingExam}
    onClose={() => {
      showExamEditor = false;
      editingExam = null;
    }}
    onChanged={() => { void loadExams(); }}
  />
  {#if view === "cards" && activeDeck}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <CardEditor
        deck={activeDeck}
        onClose={goHome}
        onStudy={() => handleStudyDeck(activeDeck!)}
        onPractice={() => handlePracticeDeck(activeDeck!)}
        onTest={() => handleTestDeck(activeDeck!)}
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
        returnLabel={activeDeck ? t("returnToDeck") : t("returnToDashboard")}
        onReview={() => (studyReviews += 1)}
        onClose={closeStudy}
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
      <div class="app-container dashboard-container py-6 sm:py-8">
        <div class="mb-5 flex min-h-9 flex-wrap items-center justify-end gap-2">
          <button onclick={() => requestNewDeck()} class="secondary-action flex items-center gap-2 px-3 py-1.5 text-xs">
            <Plus size={15} /> {t("Stapel anlegen")}
          </button>
          {#if arrangingModules}
            <button onclick={resetModuleOrder} class="secondary-action px-3 py-1.5 text-xs">{t("reset")}</button>
            <button
              onclick={() => (showModulePicker = !showModulePicker)}
              class="secondary-action px-3 py-1.5 text-xs"
              aria-expanded={showModulePicker}
            >{t("addModules")}{hiddenModules.length > 0 ? ` (${hiddenModules.length})` : ""}</button>
          {/if}
          <button
            onclick={() => {
              arrangingModules = !arrangingModules;
              finishModuleDrag(false);
              if (!arrangingModules) showModulePicker = false;
            }}
            class="{arrangingModules ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs"
          >
            {arrangingModules ? t("done") : t("arrangeDashboard")}
          </button>
        </div>

        {#if arrangingModules && showModulePicker}
          <div class="mb-5 border-y border-[#D8DEE8] py-4 dark:border-[#303744]">
            <div class="flex flex-wrap items-center gap-2">
              <span class="mr-2 text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Module")}</span>
              {#each hiddenModules as hiddenModule}
                <button onclick={() => addModule(hiddenModule)} class="secondary-action px-3 py-1.5 text-xs">
                  + {moduleTitles[hiddenModule]}
                </button>
              {/each}
              {#each hiddenDecks as deck}
                <button onclick={() => addDeckModule(deck.id)} class="secondary-action px-3 py-1.5 text-xs">
                  + {deck.name}
                </button>
              {/each}
              {#each hiddenExams as exam}
                <button onclick={() => addExamModule(exam.id)} class="secondary-action px-3 py-1.5 text-xs">
                  + {exam.name}
                </button>
              {/each}
              <button onclick={addSingleCardModule} class="secondary-action px-3 py-1.5 text-xs">+ Einzelkarte</button>
              <button onclick={() => addSpacerModule("blank")} class="secondary-action px-3 py-1.5 text-xs">+ Leerraum</button>
              <button onclick={() => addSpacerModule("divider")} class="secondary-action px-3 py-1.5 text-xs">+ Trennlinie</button>
              <button onclick={() => addSpacerModule("note")} class="secondary-action px-3 py-1.5 text-xs">+ Notizfläche</button>
            </div>
          </div>
        {/if}

        <div class="dashboard-grid" role="list" aria-label="Dashboard-Module">
          {#each moduleOrder as moduleId (moduleId)}
            <section
              role="listitem"
              data-dashboard-module={moduleId}
              data-dashboard-tone={moduleTone(moduleId)}
              data-dashboard-width={moduleWidth(moduleId)}
              data-dashboard-drop-axis={dragTargetModule === moduleId ? dragAxis : undefined}
              class="dashboard-module {moduleId === 'search' || moduleId === 'settings' ? 'dashboard-icon-module' : ''} {arrangingModules ? 'dashboard-module-arranging' : ''} {draggedModule === moduleId ? 'dashboard-module-dragging' : ''} {dragTargetModule === moduleId ? `dashboard-module-drop-target dashboard-module-drop-${dragPlacement}` : ''}"
            >
              {#if arrangingModules}
                <div class="dashboard-module-toolbar">
                  <button
                    class="dashboard-drag-handle"
                    onpointerdown={(event) => handleModulePointerDown(event, moduleId)}
                    onpointermove={(event) => handleModulePointerMove(event, moduleId)}
                    onpointerup={() => finishModuleDrag()}
                    onpointercancel={() => finishModuleDrag(false)}
                    title="{moduleTitle(moduleId)} verschieben"
                    aria-label="{moduleTitle(moduleId)} verschieben"
                  ><GripVertical size={18} /><span>{moduleTitle(moduleId)}</span></button>
                  <div class="dashboard-module-toolbar-actions">
                    {#if isDeckModule(moduleId)}
                      <div class="flex overflow-hidden rounded-md border border-current/15" role="group" aria-label="Größe des Karteikartenstapels">
                        {#each deckModuleSizes as size}
                          <button
                            class="px-2 py-1 text-[10px] font-semibold transition-colors {moduleWidth(moduleId) === size.width ? 'module-accent-fill text-white' : 'text-secondary hover:bg-current/5'}"
                            onclick={() => setDeckModuleSize(moduleId, size.width)}
                            aria-pressed={moduleWidth(moduleId) === size.width}
                          >{t(size.label)}</button>
                        {/each}
                      </div>
                    {/if}
                    <button
                      class="dashboard-order-button dashboard-move-button"
                      onclick={() => moveModuleBy(moduleId, -1)}
                      disabled={moduleOrder[0] === moduleId}
                      title="Vorheriges Modul"
                      aria-label="{moduleTitle(moduleId)} vor das vorherige Modul verschieben"
                    ><ArrowUp size={16} /></button>
                    <button
                      class="dashboard-order-button dashboard-move-button"
                      onclick={() => moveModuleBy(moduleId, 1)}
                      disabled={moduleOrder[moduleOrder.length - 1] === moduleId}
                      title="Nächstes Modul"
                      aria-label="{moduleTitle(moduleId)} hinter das nächste Modul verschieben"
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
                </div>
              {/if}

              <div class="dashboard-module-content" class:dashboard-module-content-disabled={arrangingModules}>
                {#if moduleId === "brand"}
                  <div class="surface-panel flex min-h-28 items-center gap-5 p-5 sm:p-6">
                    <div class="relative h-14 w-14 shrink-0" aria-hidden="true">
                      <span class="absolute left-1.5 top-1.5 h-12 w-12 bg-accent-correct"></span>
                      <span class="font-pixel absolute left-0 top-0 flex h-12 w-12 items-center justify-center border-[3px] border-[#111827] bg-white text-xl font-black text-[#111827] dark:border-[#F8FAFC] dark:bg-[#171B24] dark:text-[#F8FAFC]">S</span>
                      <span class="absolute left-1.5 top-1.5 h-2 w-2 bg-accent-hard"></span>
                      <span class="absolute bottom-3 right-3 h-2 w-2 bg-accent-correct"></span>
                    </div>
                    <p class="dashboard-brand-wordmark font-pixel min-w-0 text-xl font-bold text-primary dark:text-primary-dark sm:text-2xl">stapelweise</p>
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
                {:else if isDeckModule(moduleId)}
                  {@const deck = deckStore.decks.find((candidate) => candidate.id === moduleId.slice("deck:".length))}
                  {#if deck}
                    <DeckModule
                      {deck}
                      refreshToken={dashboardRevision}
                      onSelect={handleSelectDeck}
                      onStudy={handleStudyDeck}
                      onPractice={handlePracticeDeck}
                      onArchive={(selectedDeck) => (archiveConfirmDeck = selectedDeck)}
                    />
                  {/if}
                {:else if isExamModule(moduleId)}
                  {@const exam = exams.find((candidate) => candidate.id === moduleId.slice("exam:".length))}
                  {#if exam}
                    <ExamModule
                      {exam}
                      refreshToken={examRevision}
                      onStudy={(selectedExam) => handleStudyExam(selectedExam.deck_ids, selectedExam.name)}
                      onSimulate={(selectedExam) => handleSimulateExam(selectedExam.deck_ids, selectedExam.name)}
                      onEdit={requestEditExam}
                      onArchive={archiveExam}
                    />
                  {/if}
                {:else if isSpacerModule(moduleId)}
                  {@const spacer = spacerModules[moduleId] ?? { variant: "blank", note: "" }}
                  {#if spacer.variant === "blank"}
                    <div class="dashboard-spacer dashboard-spacer-blank" aria-hidden="true"></div>
                  {:else if spacer.variant === "divider"}
                    <div class="dashboard-spacer dashboard-spacer-divider" aria-hidden="true"><span></span></div>
                  {:else}
                    <div class="dashboard-spacer dashboard-spacer-note">
                      <textarea
                        data-user-content
                        value={spacer.note}
                        oninput={(event) => updateSpacerNote(moduleId, event.currentTarget.value)}
                        placeholder={t("Notiz...")}
                        aria-label={t("Notizfläche")}
                        class="h-full w-full resize-none bg-transparent p-4 text-sm text-primary outline-none placeholder:text-secondary dark:text-primary-dark"
                      ></textarea>
                    </div>
                  {/if}
                {:else if moduleId === "learning"}
                  <aside class="surface-panel h-full p-5">
                    <p class="section-kicker mb-3">{t("Lernlage")}</p>
                    {#if dashboard}
                      <div class="space-y-4">
                        <div>
                          <div class="mb-1 flex items-center justify-between text-sm">
                            <span class="font-semibold text-primary dark:text-primary-dark">{t("Lernpensum")}</span>
                            <span class="text-secondary">{dashboard.due_cards === 0 ? t("frei") : dashboard.due_cards <= 20 ? t("normal") : t("hoch")}</span>
                          </div>
                          <div class="module-accent-track h-2 overflow-hidden rounded-full">
                            <div class="module-accent-fill h-full rounded-full transition-all" style="width: {Math.min(100, dashboard.due_cards * 3)}%"></div>
                          </div>
                        </div>
                        <div>
                          <div class="mb-2 flex items-center justify-between text-sm">
                            <span class="font-semibold text-primary dark:text-primary-dark">{t("Wochenrhythmus")}</span>
                            <span class="text-secondary">{dashboard.streak_days > 0 ? t("aktiv") : t("neu starten")}</span>
                          </div>
                          <div class="grid grid-cols-7 gap-1.5">
                            {#each weekDays as day, index}
                              {@const active = index >= Math.max(0, 7 - dashboard.streak_days)}
                              <div class="flex flex-col items-center gap-1">
                                <div class="h-2 w-full rounded-full {active ? 'module-accent-fill' : 'module-accent-track'}"></div>
                                <span class="text-[10px] font-semibold text-secondary">{day}</span>
                              </div>
                            {/each}
                          </div>
                        </div>
                        <div class="grid grid-cols-2 gap-3">
                          <div class="module-accent-subpanel rounded-lg p-3">
                            <p class="text-xs text-secondary">Ø Ease</p>
                            <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{dashboard.avg_ease_factor.toFixed(2)}</p>
                          </div>
                          <div class="module-accent-subpanel rounded-lg p-3">
                            <p class="text-xs text-secondary">{t("Bibliothek")}</p>
                            <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{deckStore.decks.length}</p>
                          </div>
                        </div>
                      </div>
                    {:else}
                      <div class="grid grid-cols-3 gap-3">
                        <div class="module-accent-muted h-16 rounded-lg"></div>
                        <div class="module-accent-muted h-16 rounded-lg"></div>
                        <div class="module-accent-muted h-16 rounded-lg"></div>
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
                {:else if moduleId === "archive"}
                  <ArchiveModule
                    refreshToken={dashboardRevision + examRevision}
                    hiddenDecks={hiddenDecks}
                    hiddenExams={hiddenExams}
                    onRestore={restoreDeck}
                    onShowDeck={(deck) => addDeckModule(deck.id)}
                    onRestoreExam={restoreExam}
                    onShowExam={(exam) => addExamModule(exam.id)}
                    onEditExam={requestEditExam}
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
        onClose={closeTest}
        onStudyFailed={(cards) => {
          activeDeckIds = [];
          activeTags = [];
          activeCustomCards = cards;
          activePracticeMode = false;
          activeDeckName = `${cards.length} ${t("Falsche Testkarten")}`;
          view = "study";
        }}
      />
    </div>
  {/if}
</div>

{#if showNewDeck}
  <div in:fade={{ duration: 140 }} out:fade={{ duration: 110 }} class="fixed inset-0 z-50 flex items-center justify-center p-4" role="presentation">
    <button class="absolute inset-0 cursor-default bg-black/40 backdrop-blur-sm" onclick={() => (showNewDeck = false)} aria-label={t("Abbrechen")}></button>
    <form
      in:scale={{ duration: 180, start: 0.97, opacity: 0 }}
      out:scale={{ duration: 120, start: 0.97, opacity: 0 }}
      class="surface-panel relative z-10 w-full max-w-md p-5 shadow-elevation-high"
      onsubmit={(event) => {
        event.preventDefault();
        createDashboardDeck();
      }}
    >
      <h2 class="text-lg font-bold text-primary dark:text-primary-dark">{t("Stapel anlegen")}</h2>
      <p class="mt-1 text-sm text-secondary">{t("Gib deinem neuen Stapel einen Namen.")}</p>
      <input
        bind:value={newDeckName}
        class="mt-4 w-full rounded-md border border-[#D8DEE8] bg-white/60 px-3 py-2 text-primary outline-none focus:border-accent-correct dark:border-[#303744] dark:bg-white/5 dark:text-primary-dark"
        placeholder={t("Name des Stapels...")}
      />
      {#if newDeckError}
        <p class="mt-2 text-sm text-accent-incorrect">{newDeckError}</p>
      {/if}
      <div class="mt-5 flex justify-end gap-2">
        <button type="button" onclick={() => (showNewDeck = false)} class="secondary-action px-4 py-2 text-sm">{t("Abbrechen")}</button>
        <button type="submit" disabled={!newDeckName.trim()} class="primary-action px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-45">{t("Anlegen")}</button>
      </div>
    </form>
  </div>
{/if}

{#if archiveConfirmDeck}
  <ConfirmDialog
    title={t("Stapel archivieren")}
    message={t("Der Stapel bleibt vollständig erhalten und kann im Archiv wiederhergestellt werden.")}
    confirmLabel="Stapel archivieren"
    danger={false}
    onConfirm={() => archiveDeck(archiveConfirmDeck!)}
    onCancel={() => (archiveConfirmDeck = null)}
  />
{/if}
