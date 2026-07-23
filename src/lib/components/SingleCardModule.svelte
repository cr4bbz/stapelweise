<script lang="ts">
  import { Repeat2 } from "@lucide/svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { languageLabel } from "$lib/languages";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
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

<div class="dashboard-single-card surface-panel flex h-full flex-col p-4 sm:p-5">
  <div class="grid grid-cols-2 gap-2">
    <select
      value={selectedDeckId}
      onchange={(event) => selectDeck(event.currentTarget.value)}
      aria-label={t("Stapel auswählen")}
      class="module-accent-input min-w-0 rounded-md px-3 py-2 text-sm outline-none"
    >
      {#each decks as deck}<option value={deck.id}>{deck.name}</option>{/each}
    </select>
    <select
      value={selectedCardId}
      onchange={(event) => onSelect(event.currentTarget.value)}
      aria-label={t("Karte auswählen")}
      class="module-accent-input min-w-0 rounded-md px-3 py-2 text-sm outline-none"
    >
      <option value="">{t("Karte auswählen")}</option>
      {#each deckCards as card, index}
        <option value={card.id}>{index + 1}. {card.front.replace(/[#*_`]/g, "").slice(0, 48)}</option>
      {/each}
    </select>
  </div>

  {#if selectedCard}
    <button
      onclick={() => (showingBack = !showingBack)}
      class="module-accent-subpanel group relative mt-4 flex w-full items-center justify-center overflow-hidden rounded-lg p-4 text-center text-primary transition-colors hover:border-accent-correct dark:text-primary-dark sm:p-6"
      style="aspect-ratio: 5 / 3"
      aria-label={showingBack ? t("Vorderseite zeigen") : t("Rückseite zeigen")}
    >
      <span class="absolute left-3 top-3 text-[10px] font-semibold uppercase text-secondary">
        {showingBack ? t("Rückseite") : t("Vorderseite")}
        {selectedLanguage ? ` · ${languageLabel(selectedLanguage)}` : ""}
      </span>
      <span class="absolute right-3 top-3 text-secondary transition-colors group-hover:text-accent-correct"><Repeat2 size={18} /></span>
      <div data-user-content class="prose prose-sm max-w-full {cardFontClass}">
        {@html renderMarkdown(showingBack ? selectedCard.back : selectedCard.front)}
      </div>
    </button>
  {:else}
    <div class="module-accent-subpanel mt-4 flex min-h-40 flex-1 flex-col items-center justify-center rounded-lg border-dashed px-4 text-center">
      <p class="text-sm font-semibold text-primary dark:text-primary-dark">{t("Karte auswählen")}</p>
      <p class="mt-1 text-xs text-secondary">{t("Wähle zuerst einen Stapel und dann eine Karte.")}</p>
    </div>
  {/if}
</div>
