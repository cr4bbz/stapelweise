<script lang="ts">
  import { slide, fade } from "svelte/transition";
  import DeckList from "$lib/components/DeckList.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { DashboardStats, Deck } from "$lib/types";

  let view = $state<"decks" | "cards" | "study" | "settings">("decks");
  let activeDeck = $state<Deck | null>(null);
  let dashboard = $state<DashboardStats | null>(null);

  deckStore.load();

  $effect(() => {
    api.getDashboardStats().then((s) => (dashboard = s)).catch(() => {});
  });

  function handleOpenSettings() {
    view = "settings";
  }

  $effect(() => {
    window.addEventListener("open-settings", handleOpenSettings);
    return () => window.removeEventListener("open-settings", handleOpenSettings);
  });

  function handleSelectDeck(deck: Deck) {
    activeDeck = deck;
    view = "cards";
  }

  function handleStudyDeck(deck: Deck) {
    activeDeck = deck;
    view = "study";
  }

  function goHome() {
    view = "decks";
    activeDeck = null;
  }
</script>

{#key view}
  {#if view === "cards" && activeDeck}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
      <CardEditor
        deck={activeDeck}
        onClose={goHome}
        onStudy={() => {
          view = "study";
        }}
      />
    </div>
  {:else if view === "study" && activeDeck}
    <div transition:slide={{ duration: 200, axis: "x" }} class="h-full">
      <StudyView
        deckId={activeDeck.id}
        deckName={activeDeck.name}
        onClose={goHome}
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
      />
    </div>
  {/if}
{/key}
