<script lang="ts">
  import { slide, fade } from "svelte/transition";
  import DeckList from "$lib/components/DeckList.svelte";
  import CardEditor from "$lib/components/CardEditor.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { Deck } from "$lib/types";

  let view = $state<"decks" | "cards" | "study" | "settings">("decks");
  let activeDeck = $state<Deck | null>(null);

  deckStore.load();

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
      <DeckList
        onSelectDeck={handleSelectDeck}
        onStudyDeck={handleStudyDeck}
      />
    </div>
  {/if}
{/key}
