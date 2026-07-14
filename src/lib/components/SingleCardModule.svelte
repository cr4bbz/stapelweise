<script lang="ts">
  import { Repeat2 } from "@lucide/svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { languageLabel } from "$lib/languages";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import type { Card, Deck } from "$lib/types";

  let {
    decks = [],
    cards = [],
    selectedCardId = "",
    onSelect = (_cardId: string) => {},
  } = $props<{
    decks?: Deck[];
    cards?: Card[];
    selectedCardId?: string;
    onSelect?: (cardId: string) => void;
  }>();

  let selectedDeckId = $state("");
  let showingBack = $state(false);
  let previousCardId = "";
  let selectedCard = $derived(cards.find((card: Card) => card.id === selectedCardId) ?? null);
  let selectedLanguage = $derived(showingBack ? selectedCard?.back_language : selectedCard?.front_language);
  let deckCards = $derived(cards.filter((card: Card) => card.deck_id === selectedDeckId));
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  $effect(() => {
    void settingsStore.load();
    const selectedDeck = selectedCard?.deck_id;
    if (selectedDeck) selectedDeckId = selectedDeck;
    else if (!decks.some((deck: Deck) => deck.id === selectedDeckId)) selectedDeckId = decks[0]?.id ?? "";
  });

  $effect(() => {
    if (selectedCardId !== previousCardId) {
      previousCardId = selectedCardId;
      showingBack = false;
    }
  });

  function selectDeck(deckId: string) {
    selectedDeckId = deckId;
    if (!cards.some((card: Card) => card.id === selectedCardId && card.deck_id === deckId)) onSelect("");
  }
</script>

<div class="surface-panel flex h-full min-h-72 flex-col p-4 sm:p-5">
  <div class="grid grid-cols-2 gap-2">
    <select
      value={selectedDeckId}
      onchange={(event) => selectDeck(event.currentTarget.value)}
      aria-label="Stapel auswählen"
      class="min-w-0 rounded-md border border-[#D8DEE8] bg-white px-3 py-2 text-sm text-primary outline-none focus:border-accent-correct dark:border-[#303744] dark:bg-[#151922] dark:text-primary-dark"
    >
      {#each decks as deck}<option value={deck.id}>{deck.name}</option>{/each}
    </select>
    <select
      value={selectedCardId}
      onchange={(event) => onSelect(event.currentTarget.value)}
      aria-label="Karte auswählen"
      class="min-w-0 rounded-md border border-[#D8DEE8] bg-white px-3 py-2 text-sm text-primary outline-none focus:border-accent-correct dark:border-[#303744] dark:bg-[#151922] dark:text-primary-dark"
    >
      <option value="">Karte auswählen</option>
      {#each deckCards as card, index}
        <option value={card.id}>{index + 1}. {card.front.replace(/[#*_`]/g, "").slice(0, 48)}</option>
      {/each}
    </select>
  </div>

  {#if selectedCard}
    <button
      onclick={() => (showingBack = !showingBack)}
      class="group relative mt-4 flex min-h-52 flex-1 items-center justify-center overflow-hidden rounded-lg border border-[#D8DEE8] bg-white p-6 text-center text-primary transition-colors hover:border-accent-correct dark:border-[#303744] dark:bg-[#151922] dark:text-primary-dark"
      aria-label={showingBack ? "Vorderseite zeigen" : "Rückseite zeigen"}
    >
      <span class="absolute left-3 top-3 text-[10px] font-semibold uppercase text-secondary">
        {showingBack ? "Rückseite" : "Vorderseite"}
        {selectedLanguage ? ` · ${languageLabel(selectedLanguage)}` : ""}
      </span>
      <span class="absolute right-3 top-3 text-secondary transition-colors group-hover:text-accent-correct"><Repeat2 size={18} /></span>
      <div class="prose prose-sm max-w-full {cardFontClass}">
        {@html renderMarkdown(showingBack ? selectedCard.back : selectedCard.front)}
      </div>
    </button>
  {:else}
    <div class="mt-4 min-h-52 flex-1 rounded-lg border border-dashed border-[#D8DEE8] dark:border-[#303744]"></div>
  {/if}
</div>
