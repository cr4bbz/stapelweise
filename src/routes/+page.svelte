<script lang="ts">
  import { fade } from "svelte/transition";
  import DeckList from "$lib/components/DeckList.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import TagList from "$lib/components/TagList.svelte";
  import ExamCalendar from "$lib/components/ExamCalendar.svelte";
  import TestView from "$lib/components/TestView.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { DashboardStats, Deck } from "$lib/types";

  let view = $state<"decks" | "cards" | "study" | "search" | "settings" | "test">("decks");
  let activeDeck = $state<Deck | null>(null);
  let activeDeckIds = $state<string[]>([]);
  let activeTags = $state<string[]>([]);
  let activeCustomCards = $state<any[]>([]);
  let activeDeckName = $state("");
  let dashboard = $state<DashboardStats | null>(null);
  const weekDays = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

  let hasDecks = $derived(deckStore.decks.length > 0);
  let estimatedMinutes = $derived(dashboard ? Math.max(1, Math.ceil(dashboard.due_cards * 0.75)) : 0);
  let primaryActionLabel = $derived.by(() => {
    if (!hasDecks) return "Ersten Stapel anlegen";
    if (dashboard && dashboard.due_cards > 0) return "Lernsitzung starten";
    return "Karten pflegen";
  });
  let learningLoad = $derived.by(() => {
    if (!dashboard) return "Bereit";
    if (!hasDecks) return "Startklar";
    if (dashboard.due_cards === 0) return "Alles im Rhythmus";
    if (dashboard.due_cards <= 12) return "Kleine Runde";
    if (dashboard.due_cards <= 35) return "Guter Fokusblock";
    return "Aufholsession";
  });
  let todayNudge = $derived.by(() => {
    if (!dashboard) return "Deine Lernlage wird geladen.";
    if (!hasDecks) return "Lege deinen ersten Stapel an oder importiere vorhandene Karten. Danach zeigt Stapelweise dir hier den Tagesplan.";
    if (dashboard.due_cards === 0) return "Keine Karten drängen gerade. Perfekter Moment, um neue Karten anzulegen oder eine Prüfung zu planen.";
    if (dashboard.reviews_today > 0) return "Du bist heute schon drin. Eine weitere kurze Runde hält den Stapel leicht.";
    return "Starte mit den fälligen Karten und bring den Tag schnell auf Kurs.";
  });

  deckStore.load();

  $effect(() => {
    api.getDashboardStats().then((s) => (dashboard = s)).catch(() => {});
  });

  function handleOpenSettings() {
    view = "settings";
  }

  function handleOpenSearch() {
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
    activeDeck = deck;
    activeDeckIds = [deck.id];
    activeTags = [];
    activeDeckName = deck.name;
    view = "study";
  }

  function handleStudyDecks(decks: Deck[]) {
    activeDeckIds = decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} Stapel`;
    view = "study";
  }

  function handleStudyToday() {
    if (!dashboard || dashboard.due_cards === 0 || deckStore.decks.length === 0) return;
    activeDeckIds = deckStore.decks.map((d) => d.id);
    activeTags = [];
    activeCustomCards = [];
    activeDeckName = "Heute lernen";
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
    if (dashboard && dashboard.due_cards > 0) {
      handleStudyToday();
      return;
    }
    const firstDeck = deckStore.decks[0];
    if (firstDeck) handleSelectDeck(firstDeck);
  }

  function handleStudyTags(tags: string[]) {
    activeDeckIds = [];
    activeTags = tags;
    activeCustomCards = [];
    activeDeckName = tags.length === 1 ? `#${tags[0]}` : `${tags.length} Tags`;
    view = "study";
  }

  function handleStudyExam(deckIds: string[], examName: string) {
    activeDeckIds = deckIds;
    activeTags = [];
    activeDeckName = `Prüfung: ${examName}`;
    view = "study";
  }

  function handleSimulateExam(deckIds: string[], examName: string) {
    activeDeckIds = deckIds;
    activeTags = [];
    activeDeckName = `Simulation: ${examName}`;
    view = "test";
  }

  function goHome() {
    view = "decks";
    activeDeck = null;
    activeDeckIds = [];
    activeTags = [];
    activeDeckName = "";
  }
</script>

<div class="grid h-full w-full">
  {#if view === "cards" && activeDeck}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <CardEditor
        deck={activeDeck}
        onClose={goHome}
        onStudy={() => {
          activeDeckIds = [activeDeck!.id];
          activeDeckName = activeDeck!.name;
          view = "study";
        }}
      />
    </div>
  {:else if view === "study" && (activeDeckIds.length > 0 || activeTags.length > 0)}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <StudyView
        deckIds={activeDeckIds}
        tags={activeTags}
        customCards={activeCustomCards}
        deckName={activeDeckName}
        onClose={goHome}
      />
    </div>
  {:else if view === "search"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <SearchView
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
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <SettingsPanel onClose={goHome} />
    </div>
  {:else if view === "decks"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <div class="app-container py-6">
      {#if dashboard}
        <div class="grid grid-cols-1 gap-4 pb-6 lg:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.95fr)]">
          <section class="surface-panel overflow-hidden">
            <div class="p-5 sm:p-6">
              <div class="flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between">
                <div>
                  <p class="section-kicker mb-2">{learningLoad}</p>
                  <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">Heute lernen</h1>
                  <p class="mt-2 max-w-xl text-sm text-secondary sm:text-base">{todayNudge}</p>
                </div>
                <div class="rounded-xl border border-[#E4E7EC] bg-[#F8FAFC] px-4 py-3 text-left dark:border-[#2A303B] dark:bg-[#10131A] sm:text-right">
                  <p class="font-pixel text-2xl font-bold text-primary dark:text-primary-dark">{dashboard.due_cards}</p>
                  <p class="text-xs font-semibold uppercase tracking-wide text-secondary">Karten fällig</p>
                </div>
              </div>

              <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
                <button
                  onclick={handlePrimaryAction}
                  class="primary-action px-5 py-2.5 text-sm"
                >
                  {primaryActionLabel}
                </button>
                <button
                  onclick={requestNewExam}
                  class="secondary-action px-5 py-2.5 text-sm"
                >
                  Prüfung planen
                </button>
                <span class="text-sm text-secondary">
                  {!hasDecks
                    ? "Beginne mit einem Stapel oder Beispieldaten"
                    : dashboard.due_cards > 0
                      ? `ca. ${estimatedMinutes} Minuten · ${deckStore.decks.length} Stapel verfügbar`
                      : "Heute ist kein Pflichtblock offen"}
                </span>
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
          </section>

          <aside class="surface-panel p-5">
            <p class="section-kicker mb-3">Lernlage</p>
            <div class="space-y-4">
              <div>
                <div class="mb-1 flex items-center justify-between text-sm">
                  <span class="font-semibold text-primary dark:text-primary-dark">Lernpensum</span>
                  <span class="text-secondary">{dashboard.due_cards === 0 ? "frei" : dashboard.due_cards <= 20 ? "normal" : "hoch"}</span>
                </div>
                <div class="h-2 overflow-hidden rounded-full bg-[#E4E7EC] dark:bg-[#2A303B]">
                  <div
                    class="h-full rounded-full bg-accent-correct transition-all"
                    style="width: {Math.min(100, dashboard.due_cards * 3)}%"
                  ></div>
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
              <p class="text-sm leading-relaxed text-secondary">
                {dashboard.streak_days > 0
                  ? `Deine Serie läuft seit ${dashboard.streak_days} Tagen. Halte die heutige Runde lieber kurz als perfekt.`
                  : "Ein kurzer Start reicht, damit Stapelweise wieder einen Rhythmus erkennt."}
              </p>
            </div>
          </aside>
        </div>
      {:else}
        <div class="grid grid-cols-1 gap-4 pb-6 lg:grid-cols-[minmax(0,1.55fr)_minmax(320px,0.95fr)]">
          <section class="surface-panel p-5 sm:p-6">
            <p class="section-kicker mb-2">Lernlage</p>
            <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">Heute lernen</h1>
            <p class="mt-2 max-w-xl text-sm text-secondary sm:text-base">Deine Lernlage wird geladen. In der Desktop-App siehst du hier fällige Karten, Zeitaufwand und den direkten Start in die heutige Session.</p>
            <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
              <button onclick={requestNewDeck} class="primary-action px-5 py-2.5 text-sm">Ersten Stapel anlegen</button>
              <button onclick={requestNewExam} class="secondary-action px-5 py-2.5 text-sm">Prüfung planen</button>
              <span class="text-sm text-secondary">Lokale Lernzahlen werden geladen</span>
            </div>
          </section>
          <aside class="surface-panel p-5">
            <p class="section-kicker mb-3">Kurzüberblick</p>
            <div class="grid grid-cols-3 gap-3">
              <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
              <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
              <div class="h-16 rounded-lg bg-[#F1F5F9] dark:bg-[#10131A]"></div>
            </div>
          </aside>
        </div>
      {/if}
      <TagList onStudyTags={handleStudyTags} />
      <ExamCalendar onStudyExam={handleStudyExam} onSimulateExam={handleSimulateExam} />
      <DeckList
        onSelectDeck={handleSelectDeck}
        onStudyDeck={handleStudyDeck}
        onStudyDecks={handleStudyDecks}
      />
      </div>
    </div>
  {:else if view === "test"}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full overflow-y-auto">
      <TestView
        deckIds={activeDeckIds}
        tags={activeTags}
        testName={activeDeckName}
        onClose={goHome}
        onStudyFailed={(cards) => {
          activeDeckIds = [];
          activeTags = [];
          activeCustomCards = cards;
          activeDeckName = `${cards.length} Falsche Testkarten`;
          view = "study";
        }}
      />
    </div>
  {/if}
</div>
