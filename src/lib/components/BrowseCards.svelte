<script lang="ts">
  import type { Card } from "$lib/types";
  import FlashCard from "./FlashCard.svelte";

  let { cards, deckName = "", onClose = () => {} } = $props<{
    cards: Card[];
    deckName?: string;
    onClose?: () => void;
  }>();

  let currentIndex = $state(0);
  let isFlipped = $state(false);

  let selectedFilterTag = $state<string | null>(null);

  let allTags = $derived(
    (Array.from(new Set(cards.flatMap((c: Card) => c.tags || []))) as string[]).sort()
  );

  let activeCards = $derived(
    selectedFilterTag ? cards.filter((c: Card) => Boolean(c.tags?.includes(selectedFilterTag!))) : cards
  );

  $effect(() => {
    selectedFilterTag;
    currentIndex = 0;
    isFlipped = false;
  });

  function prev() {
    if (currentIndex > 0) {
      currentIndex--;
      isFlipped = false;
    }
  }

  function next() {
    if (currentIndex < activeCards.length - 1) {
      currentIndex++;
      isFlipped = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowLeft") {
      prev();
    } else if (e.key === "ArrowRight") {
      next();
    } else if (e.key === " " || e.key === "Spacebar") {
      e.preventDefault();
      isFlipped = !isFlipped;
    } else if (e.key === "Escape") {
      onClose();
    }
  }

  let currentCard = $derived(activeCards[currentIndex] ?? null);
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Top Bar -->
  <div class="flex flex-col gap-2 p-6 pb-2">
    <div class="flex items-center gap-3">
      <button
        onclick={onClose}
        class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
        title="Zurück (Esc)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
        </svg>
      </button>
      <h1 class="text-xl font-bold text-primary dark:text-primary-dark truncate">
        {deckName}
      </h1>
      <span class="text-secondary text-sm ml-auto">{activeCards.length > 0 ? currentIndex + 1 : 0} / {activeCards.length}</span>
    </div>

    {#if allTags.length > 0}
      <div class="flex flex-wrap items-center gap-1.5 mt-1">
        <span class="text-xs text-secondary font-medium mr-1">Tag:</span>
        <button
          onclick={() => (selectedFilterTag = null)}
          class="text-xs px-2.5 py-0.5 rounded-lg font-medium transition-all {selectedFilterTag === null ? 'bg-accent-correct text-white shadow-sm' : 'glass text-secondary hover:text-primary dark:hover:text-primary-dark'}"
        >
          Alle ({cards.length})
        </button>
        {#each allTags as tag}
          {@const count = cards.filter((c: Card) => Boolean(c.tags?.includes(tag))).length}
          <button
            onclick={() => (selectedFilterTag = selectedFilterTag === tag ? null : tag)}
            class="text-xs px-2.5 py-0.5 rounded-lg font-medium transition-all flex items-center gap-1 {selectedFilterTag === tag ? 'bg-accent-correct text-white shadow-sm' : 'glass text-secondary hover:text-primary dark:hover:text-primary-dark'}"
          >
            <span>#{tag}</span>
            <span class="text-[10px] opacity-70">({count})</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Card Area -->
  <div class="flex-1 flex flex-col items-center justify-center px-6 pb-6 gap-6">
    {#if currentCard}
      <div
        class="flex flex-col items-center gap-6 w-full cursor-pointer"
        role="button"
        tabindex="0"
        onclick={() => (isFlipped = !isFlipped)}
        onkeydown={(e) => e.key === " " && (isFlipped = !isFlipped)}
      >
        <FlashCard
          front={currentCard.front}
          back={currentCard.back}
          reasoning={currentCard.reasoning}
          tags={currentCard.tags}
          flipped={isFlipped}
          cardType={currentCard.card_type}
          content={currentCard.content}
        />
      </div>
    {/if}

    <!-- Navigation -->
    <div class="flex items-center gap-4">
      <button
        onclick={prev}
        disabled={currentIndex === 0}
        class="p-3 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors disabled:opacity-30"
        title="Zurück (←)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
        </svg>
      </button>
      <button
        onclick={() => (isFlipped = !isFlipped)}
        class="rounded-button bg-accent-correct text-white px-5 py-2 text-sm font-semibold hover:scale-[1.02] transition-transform"
      >
        {isFlipped ? "Vorderseite zeigen" : "Rückseite zeigen"}
      </button>
      <button
        onclick={next}
        disabled={currentIndex >= activeCards.length - 1}
        class="p-3 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors disabled:opacity-30"
        title="Weiter (→)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
  </div>
</div>
