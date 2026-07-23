<!--
  Dashboard container.

  Renders the modular grid from the layout store and the module registry,
  owns arrange mode (reorder / resize / add / remove modules) including the
  pointer-based drag & drop, and dispatches each slot to its self-contained
  module component. Data and navigation flow through the shared stores; only
  modal-driven actions (new deck / exam, archive confirm) come in as props.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowDown, ArrowUp, ChevronsLeft, ChevronsRight, GripVertical, Plus, X } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { dashboardLayout } from "$lib/dashboard/layout.svelte";
  import {
    deckIdOf,
    examIdOf,
    isDeckModule,
    isExamModule,
    isIconModule,
    isRemovable,
    isSingleCardModule,
    isSpacerModule,
    moduleRegistry,
    moduleTone,
    moduleWidthOptions,
    spacerTitle,
    type DashboardModuleKey,
  } from "$lib/dashboard/registry";
  import { t } from "$lib/i18n";
  import type { Card, Deck, Exam } from "$lib/types";

  import BrandModule from "$lib/components/dashboard/BrandModule.svelte";
  import SearchModule from "$lib/components/dashboard/SearchModule.svelte";
  import SettingsModule from "$lib/components/dashboard/SettingsModule.svelte";
  import FocusModule from "$lib/components/dashboard/FocusModule.svelte";
  import LearningModule from "$lib/components/dashboard/LearningModule.svelte";
  import ContinueModule from "$lib/components/dashboard/ContinueModule.svelte";
  import TimerModule from "$lib/components/dashboard/TimerModule.svelte";
  import ProblemsModule from "$lib/components/dashboard/ProblemsModule.svelte";
  import WeekPlanModule from "$lib/components/dashboard/WeekPlanModule.svelte";
  import QuickCaptureModule from "$lib/components/dashboard/QuickCaptureModule.svelte";
  import LearningTimeModule from "$lib/components/dashboard/LearningTimeModule.svelte";
  import MilestonesModule from "$lib/components/dashboard/MilestonesModule.svelte";
  import SpacerModule from "$lib/components/dashboard/SpacerModule.svelte";
  import DeckModule from "$lib/components/DeckModule.svelte";
  import ExamModule from "$lib/components/ExamModule.svelte";
  import SingleCardModule from "$lib/components/SingleCardModule.svelte";
  import ArchiveModule from "$lib/components/ArchiveModule.svelte";
  import TagList from "$lib/components/TagList.svelte";

  let {
    exams = [],
    examsLoaded = false,
    examRevision = 0,
    onRequestNewDeck,
    onRequestNewExam,
    onEditExam,
    onArchiveDeck,
    onArchiveExam,
    onRestoreDeck,
    onRestoreExam,
  } = $props<{
    exams?: Exam[];
    examsLoaded?: boolean;
    examRevision?: number;
    onRequestNewDeck: () => void;
    onRequestNewExam: () => void;
    onEditExam: (exam: Exam) => void;
    onArchiveDeck: (deck: Deck) => void;
    onArchiveExam: (exam: Exam) => void;
    onRestoreDeck: (deck: Deck) => void;
    onRestoreExam: (exam: Exam) => void;
  }>();

  type ModuleDragCandidate = { moduleId: DashboardModuleKey; startX: number; startY: number; pointerId: number };
  type ModuleDropAxis = "inline" | "block";

  let arrangingModules = $state(false);
  let showModulePicker = $state(false);
  let draggedModule = $state<DashboardModuleKey | null>(null);
  let dragCandidate = $state<ModuleDragCandidate | null>(null);
  let dragTargetModule = $state<DashboardModuleKey | null>(null);
  let dragPlacement = $state<"before" | "after">("before");
  let dragAxis = $state<ModuleDropAxis>("inline");

  let dashboardCards = $state<Card[]>([]);
  let dashboardCardSignature = "";

  let hiddenDecks = $derived(deckStore.decks.filter((deck) => dashboardLayout.hiddenDeckIds.includes(deck.id)));
  let hiddenExams = $derived(exams.filter((exam: Exam) => dashboardLayout.hiddenExamIds.includes(exam.id)));

  const deckById = (id: string) => deckStore.decks.find((deck) => deck.id === id);
  const examById = (id: string) => exams.find((exam: Exam) => exam.id === id);

  function moduleTitle(moduleId: DashboardModuleKey): string {
    if (isSingleCardModule(moduleId)) return t("Einzelkarte");
    if (isDeckModule(moduleId)) return deckById(deckIdOf(moduleId))?.name ?? t("Stapel");
    if (isExamModule(moduleId)) return examById(examIdOf(moduleId))?.name ?? t("Prüfung");
    if (isSpacerModule(moduleId)) return t(spacerTitle(dashboardLayout.spacers[moduleId]?.variant ?? "blank"));
    return t(moduleRegistry[moduleId].titleKey);
  }

  // ── Keep dashboard data in sync ─────────────────────────────

  $effect(() => {
    dashboardLayout.reconcile(deckStore.decks, exams, examsLoaded);
  });

  $effect(() => {
    if (!dashboardLayout.order.some(isSingleCardModule)) return;
    const signature = `${deckStore.decks.map((deck) => deck.id).join(",")}:${dashboardStore.revision}`;
    if (!deckStore.decks.length || dashboardCardSignature === signature) return;
    dashboardCardSignature = signature;
    Promise.all(deckStore.decks.map((deck) => api.listCards(deck.id)))
      .then((cards) => (dashboardCards = cards.flat()))
      .catch(() => (dashboardCards = []));
  });

  // ── Drag & drop reordering ──────────────────────────────────

  function handlePointerDown(event: PointerEvent, moduleId: DashboardModuleKey) {
    if (!arrangingModules || event.button !== 0) return;
    dragCandidate = { moduleId, startX: event.clientX, startY: event.clientY, pointerId: event.pointerId };
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: PointerEvent, moduleId: DashboardModuleKey) {
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
    const sourceWidth = dashboardLayout.widthOf(draggedModule);
    const targetWidth = dashboardLayout.widthOf(dragTargetModule);
    const columns = window.matchMedia("(min-width: 1024px)").matches ? 12 : window.matchMedia("(min-width: 640px)").matches ? 6 : 2;
    dragAxis = columns > 2 && sourceWidth + targetWidth <= columns ? "inline" : "block";
    dragPlacement = dragAxis === "inline"
      ? event.clientX < rect.left + rect.width / 2 ? "before" : "after"
      : event.clientY < rect.top + rect.height / 2 ? "before" : "after";
  }

  function finishDrag(commit = true) {
    if (commit && draggedModule && dragTargetModule) dashboardLayout.move(draggedModule, dragTargetModule, dragPlacement);
    dragCandidate = null;
    draggedModule = null;
    dragTargetModule = null;
    dragPlacement = "before";
    dragAxis = "inline";
  }

  function toggleArranging() {
    arrangingModules = !arrangingModules;
    finishDrag(false);
    if (!arrangingModules) showModulePicker = false;
  }

  function resetDashboard() {
    dashboardLayout.reset();
    showModulePicker = false;
  }

  onMount(() => finishDrag(false));
</script>

<div class="app-container dashboard-container py-6 sm:py-8">
  <div class="mb-5 flex min-h-9 flex-wrap items-center justify-end gap-2">
    <button onclick={onRequestNewDeck} class="secondary-action flex items-center gap-2 px-3 py-1.5 text-xs">
      <Plus size={15} /> {t("Stapel anlegen")}
    </button>
    {#if arrangingModules}
      <button onclick={resetDashboard} class="secondary-action px-3 py-1.5 text-xs">{t("reset")}</button>
      <button
        onclick={() => (showModulePicker = !showModulePicker)}
        class="secondary-action px-3 py-1.5 text-xs"
        aria-expanded={showModulePicker}
      >{t("addModules")}{dashboardLayout.hiddenBuiltins.length > 0 ? ` (${dashboardLayout.hiddenBuiltins.length})` : ""}</button>
    {/if}
    <button
      onclick={toggleArranging}
      class="{arrangingModules ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs"
    >{arrangingModules ? t("done") : t("arrangeDashboard")}</button>
  </div>

  {#if arrangingModules && showModulePicker}
    <div class="mb-5 border-y border-[#D8DEE8] py-4 dark:border-[#303744]">
      <div class="flex flex-wrap items-center gap-2">
        <span class="mr-2 text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Module")}</span>
        {#each dashboardLayout.hiddenBuiltins as hiddenModule}
          <button onclick={() => dashboardLayout.addBuiltin(hiddenModule)} class="secondary-action px-3 py-1.5 text-xs">
            + {t(moduleRegistry[hiddenModule].titleKey)}
          </button>
        {/each}
        {#each hiddenDecks as deck}
          <button onclick={() => dashboardLayout.addDeck(deck.id)} class="secondary-action px-3 py-1.5 text-xs">+ {deck.name}</button>
        {/each}
        {#each hiddenExams as exam}
          <button onclick={() => dashboardLayout.addExam(exam.id)} class="secondary-action px-3 py-1.5 text-xs">+ {exam.name}</button>
        {/each}
        <button onclick={() => dashboardLayout.addSingleCard()} class="secondary-action px-3 py-1.5 text-xs">+ {t("Einzelkarte")}</button>
        <button onclick={() => dashboardLayout.addSpacer("blank")} class="secondary-action px-3 py-1.5 text-xs">+ {t("Leerraum")}</button>
        <button onclick={() => dashboardLayout.addSpacer("divider")} class="secondary-action px-3 py-1.5 text-xs">+ {t("Trennlinie")}</button>
        <button onclick={() => dashboardLayout.addSpacer("note")} class="secondary-action px-3 py-1.5 text-xs">+ {t("Notizfläche")}</button>
      </div>
    </div>
  {/if}

  <div class="dashboard-grid" role="list" aria-label="Dashboard-Module">
    {#each dashboardLayout.order as moduleId (moduleId)}
      <section
        role="listitem"
        data-dashboard-module={moduleId}
        data-dashboard-tone={moduleTone(moduleId)}
        data-dashboard-width={dashboardLayout.widthOf(moduleId)}
        data-dashboard-drop-axis={dragTargetModule === moduleId ? dragAxis : undefined}
        class="dashboard-module {isIconModule(moduleId) ? 'dashboard-icon-module' : ''} {arrangingModules ? 'dashboard-module-arranging' : ''} {draggedModule === moduleId ? 'dashboard-module-dragging' : ''} {dragTargetModule === moduleId ? `dashboard-module-drop-target dashboard-module-drop-${dragPlacement}` : ''}"
      >
        {#if arrangingModules}
          <div class="dashboard-module-toolbar">
            <button
              class="dashboard-drag-handle"
              onpointerdown={(event) => handlePointerDown(event, moduleId)}
              onpointermove={(event) => handlePointerMove(event, moduleId)}
              onpointerup={() => finishDrag()}
              onpointercancel={() => finishDrag(false)}
              title="{moduleTitle(moduleId)} verschieben"
              aria-label="{moduleTitle(moduleId)} verschieben"
            ><GripVertical size={18} /><span>{moduleTitle(moduleId)}</span></button>
            <div class="dashboard-module-toolbar-actions">
              <button
                class="dashboard-order-button dashboard-resize-button"
                onclick={() => dashboardLayout.resize(moduleId, -1)}
                disabled={moduleWidthOptions(moduleId).indexOf(dashboardLayout.widthOf(moduleId)) === 0}
                title="Schmaler"
                aria-label="{moduleTitle(moduleId)} schmaler machen"
              ><ChevronsLeft size={16} /></button>
              <button
                class="dashboard-order-button dashboard-resize-button"
                onclick={() => dashboardLayout.resize(moduleId, 1)}
                disabled={moduleWidthOptions(moduleId).indexOf(dashboardLayout.widthOf(moduleId)) === moduleWidthOptions(moduleId).length - 1}
                title="Breiter"
                aria-label="{moduleTitle(moduleId)} breiter machen"
              ><ChevronsRight size={16} /></button>
              <button
                class="dashboard-order-button dashboard-move-button"
                onclick={() => dashboardLayout.moveBy(moduleId, -1)}
                disabled={dashboardLayout.order[0] === moduleId}
                title="Vorheriges Modul"
                aria-label="{moduleTitle(moduleId)} vor das vorherige Modul verschieben"
              ><ArrowUp size={16} /></button>
              <button
                class="dashboard-order-button dashboard-move-button"
                onclick={() => dashboardLayout.moveBy(moduleId, 1)}
                disabled={dashboardLayout.order[dashboardLayout.order.length - 1] === moduleId}
                title="Nächstes Modul"
                aria-label="{moduleTitle(moduleId)} hinter das nächste Modul verschieben"
              ><ArrowDown size={16} /></button>
              {#if isRemovable(moduleId)}
                <button
                  class="dashboard-order-button dashboard-remove-button"
                  onclick={() => dashboardLayout.remove(moduleId)}
                  title="Entfernen"
                  aria-label="{moduleTitle(moduleId)} vom Dashboard entfernen"
                ><X size={16} /></button>
              {/if}
            </div>
          </div>
        {/if}

        <div class="dashboard-module-content" class:dashboard-module-content-disabled={arrangingModules}>
          {#if moduleId === "brand"}
            <BrandModule />
          {:else if moduleId === "search"}
            <SearchModule />
          {:else if moduleId === "settings"}
            <SettingsModule />
          {:else if moduleId === "focus"}
            <FocusModule {onRequestNewDeck} {onRequestNewExam} />
          {:else if moduleId === "learning"}
            <LearningModule />
          {:else if moduleId === "continue"}
            <ContinueModule />
          {:else if moduleId === "timer"}
            <TimerModule />
          {:else if moduleId === "problems"}
            <ProblemsModule />
          {:else if moduleId === "weekPlan"}
            <WeekPlanModule />
          {:else if moduleId === "quickCapture"}
            <QuickCaptureModule />
          {:else if moduleId === "learningTime"}
            <LearningTimeModule />
          {:else if moduleId === "milestones"}
            <MilestonesModule />
          {:else if moduleId === "tags"}
            <TagList onStudyTags={(tags) => navigation.studyTags(tags)} />
          {:else if moduleId === "archive"}
            <ArchiveModule
              refreshToken={dashboardStore.revision + examRevision}
              {hiddenDecks}
              {hiddenExams}
              onRestore={onRestoreDeck}
              onShowDeck={(deck) => dashboardLayout.addDeck(deck.id)}
              onRestoreExam={onRestoreExam}
              onShowExam={(exam) => dashboardLayout.addExam(exam.id)}
              onEditExam={onEditExam}
            />
          {:else if isSingleCardModule(moduleId)}
            <SingleCardModule
              decks={deckStore.decks}
              cards={dashboardCards}
              selectedCardId={dashboardLayout.singleCards[moduleId] ?? ""}
              onSelect={(cardId) => dashboardLayout.selectSingleCard(moduleId, cardId)}
            />
          {:else if isDeckModule(moduleId)}
            {@const deck = deckById(deckIdOf(moduleId))}
            {#if deck}
              <DeckModule
                {deck}
                refreshToken={dashboardStore.revision}
                onSelect={(selected) => navigation.openDeck(selected)}
                onStudy={(selected) => navigation.studyDeck(selected)}
                onPractice={(selected) => navigation.practiceDeck(selected)}
                onArchive={onArchiveDeck}
              />
            {/if}
          {:else if isExamModule(moduleId)}
            {@const exam = examById(examIdOf(moduleId))}
            {#if exam}
              <ExamModule
                {exam}
                refreshToken={examRevision}
                onStudy={(selected) => navigation.studyExam(selected.deck_ids, selected.name)}
                onSimulate={(selected) => navigation.simulateExam(selected.deck_ids, selected.name)}
                onEdit={onEditExam}
                onArchive={onArchiveExam}
              />
            {/if}
          {:else if isSpacerModule(moduleId)}
            {@const spacer = dashboardLayout.spacers[moduleId] ?? { variant: "blank", note: "" }}
            <SpacerModule
              variant={spacer.variant}
              note={spacer.note}
              onNoteChange={(note) => dashboardLayout.updateSpacerNote(moduleId, note)}
            />
          {/if}
        </div>
      </section>
    {/each}
  </div>

  {#if dashboardLayout.order.length === 0}
    <div class="flex min-h-72 flex-col items-center justify-center text-center">
      <p class="font-pixel text-lg text-primary dark:text-primary-dark">{t("Dein Dashboard ist leer")}</p>
      <p class="mt-3 max-w-md text-sm text-secondary">{t("Füge genau die Module hinzu, die du beim Lernen sehen möchtest.")}</p>
      <button
        onclick={() => { arrangingModules = true; showModulePicker = true; }}
        class="primary-action mt-5 px-4 py-2 text-sm"
      >{t("Module hinzufügen")}</button>
    </div>
  {/if}
</div>
