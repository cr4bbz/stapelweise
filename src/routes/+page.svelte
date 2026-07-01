<script lang="ts">
  import { slide, fade } from "svelte/transition";
  import DeckList from "$lib/components/DeckList.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SearchView from "$lib/components/SearchView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { DashboardStats, Deck } from "$lib/types";

  let view = $state<"decks" | "cards" | "study" | "search" | "settings">("decks");
  let activeDeck = $state<Deck | null>(null);
  let activeDeckIds = $state<string[]>([]);
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
    activeDeckName = deck.name;
    view = "study";
  }

  function handleStudyDecks(decks: Deck[]) {
    activeDeckIds = decks.map((d) => d.id);
    activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} Stapel`;
    view = "study";
  }

  function goHome() {
    view = "decks";
    activeDeck = null;
    activeDeckIds = [];
    activeDeckName = "";
  }
</script>

{#key view}
  {#if view === "cards" && activeDeck}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
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
  {:else if view === "study" && activeDeckIds.length > 0}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
      <StudyView
        deckIds={activeDeckIds}
        deckName={activeDeckName}
        onClose={goHome}
      />
    </div>
  {:else if view === "search"}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
      <SearchView
        onClose={goHome}
        onSelectCard={(deckId: string) => {
          const deck = deckStore.decks.find((d) => d.id === deckId);
          if (deck) {
            activeDeck = deck;
            activeDeckIds = [];
            activeDeckName = "";
            view = "cards";
          }
        }}
      />
    </div>
  {:else if view === "settings"}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
      <SettingsPanel onClose={goHome} />
    </div>
  {:else}
    <div transition:fade={{ duration: 150 }} class="h-full">
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
      <DeckList
        onSelectDeck={handleSelectDeck}
        onStudyDeck={handleStudyDeck}
        onStudyDecks={handleStudyDecks}
      />
    </div>
  {/if}
{/key}
