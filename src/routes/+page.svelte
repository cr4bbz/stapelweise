<script lang="ts">
  import DeckList from "$lib/components/DeckList.svelte";
  import StudyView from "$lib/components/StudyView.svelte";
  import { deckStore } from "$lib/stores/decks.svelte";
  import type { Deck } from "$lib/types";

  let view = $state<"decks" | "study">("decks");
  let activeDeck = $state<Deck | null>(null);

  deckStore.load();

  function startStudy(deck: Deck) {
    activeDeck = deck;
    view = "study";
  }

  function closeStudy() {
    view = "decks";
    activeDeck = null;
  }

  // Listen for hash-based navigation from DeckList
  $effect(() => {
    function handleHash() {
      const hash = window.location.hash;
      const match = hash.match(/^#\/study\/(.+)$/);
      if (match) {
        const deckId = match[1];
        const deck = deckStore.decks.find((d) => d.id === deckId);
        if (deck) {
          startStudy(deck);
        }
        window.location.hash = "";
      }
    }
    window.addEventListener("hashchange", handleHash);
    // Check on initial load
    handleHash();
    return () => window.removeEventListener("hashchange", handleHash);
  });
</script>

{#if view === "study" && activeDeck}
  <StudyView deckId={activeDeck.id} deckName={activeDeck.name} onClose={closeStudy} />
{:else}
  <DeckList />
{/if}
