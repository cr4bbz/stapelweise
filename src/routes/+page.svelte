<!--
  Application shell.

  Routes between the top-level views (dashboard, deck editor, study, search,
  settings, test) based on the navigation store, mounts the modular Dashboard,
  and owns the modal-driven flows that don't belong to any single module:
  creating a deck or exam, confirming a deck archive, and handling deep links.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import TestView from "$lib/components/TestView.svelte";
  import ExamCalendar from "$lib/components/ExamCalendar.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { dashboardLayout } from "$lib/dashboard/layout.svelte";
  import { listenForDeepLinks, type StapelweiseDeepLink } from "$lib/deep-link";
  import type { Deck, Exam } from "$lib/types";
  import { t } from "$lib/i18n";

  const fadeIn = { duration: 150 };
  const fadeOut = { duration: 100 };
  const viewClasses = "app-scroll col-start-1 row-start-1 h-full w-full overflow-y-auto";

  let exams = $state<Exam[]>([]);
  let examsLoaded = $state(false);
  let examRevision = $state(0);
  let showExamEditor = $state(false);
  let editingExam = $state<Exam | null>(null);
  let showNewDeck = $state(false);
  let newDeckName = $state("");
  let newDeckError = $state<string | null>(null);
  let archiveConfirmDeck = $state<Deck | null>(null);

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
    dashboardLayout.load();
    void dashboardStore.refresh();
    void loadExams();
  });

  onMount(() => {
    let unlisten = () => {};
    listenForDeepLinks(handleDeepLink).then((dispose) => (unlisten = dispose));
    return () => unlisten();
  });

  onMount(() => {
    const openSettings = () => navigation.openSettings();
    const openSearch = () => navigation.openSearch();
    window.addEventListener("open-settings", openSettings);
    window.addEventListener("open-search", openSearch);
    return () => {
      window.removeEventListener("open-settings", openSettings);
      window.removeEventListener("open-search", openSearch);
    };
  });

  // ── Deck & exam modal flows ─────────────────────────────────

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
      dashboardStore.invalidate();
    } catch {
      newDeckError = t("Fehler beim Erstellen des Stapels");
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

  async function archiveDeck(deck: Deck) {
    try {
      await deckStore.archive(deck.id);
      dashboardLayout.relocateBeforeArchive(`deck:${deck.id}`);
      dashboardStore.invalidate();
    } finally {
      archiveConfirmDeck = null;
    }
  }

  async function restoreDeck(deck: Deck) {
    await deckStore.restore(deck.id);
    dashboardLayout.addDeck(deck.id);
    dashboardStore.invalidate();
  }

  async function archiveExam(exam: Exam) {
    await api.archiveExam(exam.id);
    dashboardLayout.relocateBeforeArchive(`exam:${exam.id}`);
    await loadExams();
  }

  async function restoreExam(exam: Exam) {
    await api.restoreExam(exam.id);
    await loadExams();
    dashboardLayout.addExam(exam.id);
  }

  async function handleDeepLink(link: StapelweiseDeepLink) {
    await deckStore.load();
    if (link.kind === "new-deck") {
      navigation.openDashboard();
      requestNewDeck(link.name ?? "");
      return;
    }
    const deck = deckStore.decks.find(
      (candidate) => candidate.id === link.deck || candidate.name.toLowerCase() === link.deck.toLowerCase(),
    );
    if (!deck) return;
    navigation.openDeck(deck);
    if (link.kind === "new-card") {
      await Promise.resolve();
      window.dispatchEvent(new CustomEvent("stapelweise:prefill-card", { detail: { deckId: deck.id, front: link.front, back: link.back } }));
    }
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

  {#if navigation.view === "cards" && navigation.activeDeck}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <CardEditor
        deck={navigation.activeDeck}
        onClose={navigation.goHome}
        onStudy={() => navigation.studyDeck(navigation.activeDeck!)}
        onPractice={() => navigation.practiceDeck(navigation.activeDeck!)}
      />
    </div>
  {:else if navigation.view === "study" && navigation.hasActiveStudy}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <StudyView
        deckIds={navigation.activeDeckIds}
        tags={navigation.activeTags}
        customCards={navigation.activeCustomCards}
        deckName={navigation.activeDeckName}
        practiceMode={navigation.activePracticeMode}
        returnLabel={navigation.activeDeck ? t("returnToDeck") : t("returnToDashboard")}
        onReview={navigation.recordReview}
        onClose={navigation.closeStudy}
      />
    </div>
  {:else if navigation.view === "search"}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <SearchView
        initialQuery={navigation.searchQuery}
        onClose={navigation.goHome}
        onSelectCard={(deckId: string) => navigation.openDeckById(deckId)}
      />
    </div>
  {:else if navigation.view === "settings"}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <SettingsPanel onClose={navigation.goHome} />
    </div>
  {:else if navigation.view === "decks"}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <Dashboard
        {exams}
        {examsLoaded}
        {examRevision}
        onRequestNewDeck={() => requestNewDeck()}
        onRequestNewExam={requestNewExam}
        onEditExam={requestEditExam}
        onArchiveDeck={(deck) => (archiveConfirmDeck = deck)}
        onArchiveExam={archiveExam}
        onRestoreDeck={restoreDeck}
        onRestoreExam={restoreExam}
      />
    </div>
  {:else if navigation.view === "test"}
    <div in:fade={fadeIn} out:fade={fadeOut} class={viewClasses}>
      <TestView
        deckIds={navigation.activeDeckIds}
        tags={navigation.activeTags}
        testName={navigation.activeDeckName}
        onClose={navigation.goHome}
        onStudyFailed={(cards) => navigation.studyTestFailures(cards)}
      />
    </div>
  {/if}
</div>

{#if showNewDeck}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4" role="presentation">
    <button class="absolute inset-0 cursor-default bg-black/40 backdrop-blur-sm" onclick={() => (showNewDeck = false)} aria-label={t("Abbrechen")}></button>
    <form
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
