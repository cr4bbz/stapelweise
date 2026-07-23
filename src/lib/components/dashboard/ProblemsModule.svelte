<!-- Problem-cards module: surfaces cards that are struggling (low streak or low
     ease) and lets the user drill them in a targeted round. -->
<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { t } from "$lib/i18n";
  import type { Card, CardState } from "$lib/types";

  let problemCards = $state<Card[]>([]);
  let problemLoading = $state(false);
  let loadedKey = "";

  async function loadProblemCards() {
    if (deckStore.decks.length === 0) {
      problemCards = [];
      return;
    }
    problemLoading = true;
    try {
      const cards = (await Promise.all(deckStore.decks.map((deck) => api.listCards(deck.id)))).flat();
      const states = await Promise.all(cards.map((card) => api.getCardState(card.id)));
      problemCards = cards
        .map((card, index) => ({ card, state: states[index] }))
        .filter((entry): entry is { card: Card; state: CardState } => Boolean(entry.state))
        .filter(({ state }) => state.total_reviews > 0 && (state.correct_streak < 2 || state.ease_factor < 2.3))
        .sort((a, b) => a.state.correct_streak - b.state.correct_streak || a.state.ease_factor - b.state.ease_factor)
        .slice(0, 20)
        .map(({ card }) => card);
    } finally {
      problemLoading = false;
    }
  }

  $effect(() => {
    const key = `${deckStore.decks.map((deck) => deck.id).join(",")}:${dashboardStore.revision}`;
    if (!key || key === loadedKey) return;
    loadedKey = key;
    void loadProblemCards();
  });
</script>

<div class="surface-panel h-full p-5">
  <div class="flex items-start justify-between gap-4">
    <p class="section-kicker mb-2">{t("Problemkarten")}</p>
    <span class="font-pixel text-lg text-primary dark:text-primary-dark">{problemLoading ? "..." : problemCards.length}</span>
  </div>
  <button
    onclick={() => navigation.studyCards(problemCards, t("Problemkarten"))}
    disabled={problemLoading || problemCards.length === 0}
    class="primary-action mt-5 px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-40"
  >{t("Gezielt üben")}</button>
</div>
