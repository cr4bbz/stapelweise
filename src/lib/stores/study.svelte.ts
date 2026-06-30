import * as api from "$lib/api";
import type { DueCard, CardState } from "$lib/types";

let dueCards = $state<DueCard[]>([]);
let currentIndex = $state(0);
let isFlipped = $state(false);
let sessionActive = $state(false);
let dueCount = $state(0);
let totalCount = $state(0);

function reset() {
  dueCards = [];
  currentIndex = 0;
  isFlipped = false;
  sessionActive = false;
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
    get progress(): number {
      if (dueCards.length === 0) return 0;
      return (currentIndex + (isFlipped ? 1 : 0)) / (dueCards.length + 1);
    },

    async startSession(deckId: string) {
      const cards = await api.getDueCards(deckId, 50);
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

        // If the card failed, re-add it at the end
        if (quality < 3) {
          dueCards = [
            ...dueCards,
            { card: card.card, state: newState },
          ];
        }
      }

      // Adjust index
      if (currentIndex >= dueCards.length) {
        currentIndex = dueCards.length > 0 ? 0 : 0;
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

    endSession() {
      reset();
    },
  };
}

const studyStore = getStudyStore();
export { studyStore };
