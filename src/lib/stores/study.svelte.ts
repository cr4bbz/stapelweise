import * as api from "$lib/api";
import type { DueCard, CardState } from "$lib/types";
import { settingsStore } from "./settings.svelte";

let dueCards = $state<DueCard[]>([]);
let currentIndex = $state(0);
let isFlipped = $state(false);
let sessionActive = $state(false);
let dueCount = $state(0);
let totalCount = $state(0);

let currentDeckId = $state<string | null>(null);

function reset() {
  dueCards = [];
  currentIndex = 0;
  isFlipped = false;
  sessionActive = false;
  currentDeckId = null;
}

export function getStudyStore() {
  return {
    get dueCards() {
      return dueCards;
    },
    get currentCard(): DueCard | null {
      return dueCards[currentIndex] ?? null;
    },
    get currentIndex() {
      return currentIndex;
    },
    get isFlipped() {
      return isFlipped;
    },
    get sessionActive() {
      return sessionActive;
    },
    get dueCount() {
      return dueCount;
    },
    get totalCount() {
      return totalCount;
    },
    async startCustomSession(cards: any[]) {
      reset();
      if (cards.length === 0) return false;

      // Map Card to DueCard format (pairing card with dummy/current state)
      const dueCardsList = await Promise.all(
        cards.map(async (card) => {
          try {
            const state = await api.getCardState(card.id);
            return [card, state] as [any, any];
          } catch {
            return [card, { interval: 0, ease_factor: 2.5, repetitions: 0, next_review: "", total_reviews: 0, correct_streak: 0, last_review: null }] as [any, any];
          }
        })
      );

      dueCards = dueCardsList as unknown as DueCard[];
      sessionActive = true;
      return true;
    },

    async startSession(deckIds: string[], limit: number = 50) {
      currentDeckId = deckIds.length === 1 ? deckIds[0] : null;
      const cards = await api.getDueCards(deckIds, limit);
      if (cards.length === 0) return false;
      dueCards = cards;
      currentIndex = 0;
      isFlipped = false;
      sessionActive = true;
      return true;
    },

    async startSessionByTags(tags: string[], limit: number = 50) {
      currentDeckId = null;
      const cards = await api.getDueCardsByTags(tags, limit);
      if (cards.length === 0) return false;
      dueCards = cards;
      currentIndex = 0;
      isFlipped = false;
      sessionActive = true;
      return true;
    },

    flip() {
      isFlipped = true;
    },

    async rate(quality: number) {
      const card = this.currentCard;
      if (!card) return;

      const newState: CardState = await api.submitReview(card.card.id, quality);

      // Update local state
      const idx = dueCards.findIndex((c) => c.card.id === card.card.id);
      if (idx !== -1) {
        // Remove card from session (it's been reviewed)
        dueCards = [...dueCards.slice(0, idx), ...dueCards.slice(idx + 1)];

        // If the card failed (below pass threshold), re-add it at the end
        if (quality < settingsStore.current.sm2_pass_threshold) {
          dueCards = [
            ...dueCards,
            { card: card.card, state: newState },
          ];
        }
      }

      // Adjust index
      if (currentIndex >= dueCards.length) {
        currentIndex = 0;
      }

      isFlipped = false;

      // Check if session is over
      if (dueCards.length === 0) {
        sessionActive = false;
      }
    },

    async loadCounts(deckId: string) {
      [dueCount, totalCount] = await Promise.all([
        api.countDueCards(deckId),
        api.countTotalCards(deckId),
      ]);
    },

    async undo() {
      if (!currentDeckId) return;
      const dueCard = await api.undoLastReview(currentDeckId);
      if (dueCard) {
        // Re-insert undone card at the current position
        dueCards = [
          ...dueCards.slice(0, currentIndex),
          dueCard,
          ...dueCards.slice(currentIndex),
        ];
        sessionActive = true;
        isFlipped = false;
      }
    },

    endSession() {
      reset();
    },
  };
}

const studyStore = getStudyStore();
export { studyStore };
