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
    activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} Stapel`;
    view = "study";
  }

  function handleStudyTags(tags: string[]) {
    activeDeckIds = [];
    activeTags = tags;
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
      {#if dashboard}
        <div class="grid grid-cols-3 gap-4 px-6 pt-6 pb-2">
          <div class="glass rounded-card p-4 text-center">
            <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Fällig</p>
            <p class="text-2xl font-bold text-accent-correct dark:text-accent-correct-dark">{dashboard.due_cards}</p>
          </div>
          <div class="glass rounded-card p-4 text-center">
            <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Heute gelernt</p>
            <p class="text-2xl font-bold text-accent-easy dark:text-accent-easy-dark">{dashboard.reviews_today}</p>
          </div>
          <div class="glass rounded-card p-4 text-center">
            <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Streak</p>
            <p class="text-2xl font-bold text-accent-hard dark:text-accent-hard-dark">{dashboard.streak_days} Tage</p>
          </div>
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
