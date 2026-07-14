import * as api from "$lib/api";
import type { Card, DueCard, CardState } from "$lib/types";
import { settingsStore } from "./settings.svelte";

let dueCards = $state<DueCard[]>([]);
let currentIndex = $state(0);
let isFlipped = $state(false);
let sessionActive = $state(false);
let dueCount = $state(0);
let totalCount = $state(0);
let practiceMode = $state(false);
let sessionSize = $state(0);
let completedCount = $state(0);
let sessionKey = $state<string | null>(null);

let currentDeckId = $state<string | null>(null);

function reset() {
  dueCards = [];
  currentIndex = 0;
  isFlipped = false;
  sessionActive = false;
  currentDeckId = null;
  practiceMode = false;
  sessionSize = 0;
  completedCount = 0;
  sessionKey = null;
}

function shuffleCards(cards: Card[]): Card[] {
  const shuffled = [...cards];
  for (let i = shuffled.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled;
}

async function toDueCards(cards: Card[]): Promise<DueCard[]> {
  return Promise.all(
    cards.map(async (card) => {
      try {
        const state = await api.getCardState(card.id);
        if (state) return { card, state };
      } catch {
        // A practice round can still run if a state cannot be loaded.
      }
      return {
        card,
        state: {
          card_id: card.id,
          interval: 0,
          ease_factor: 2.5,
          repetitions: 0,
          next_review: "",
          total_reviews: 0,
          correct_streak: 0,
          last_review: null,
        },
      };
    })
  );
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
    get isPractice() {
      return practiceMode;
    },
    get sessionSize() {
      return sessionSize;
    },
    get completedCount() {
      return completedCount;
    },
    canResumeSession(key: string) {
      return sessionActive && sessionKey === key && dueCards.length > 0;
    },
    async startCustomSession(cards: Card[], key: string | null = null) {
      reset();
      if (cards.length === 0) return false;
      dueCards = await toDueCards(cards);
      sessionSize = dueCards.length;
      sessionKey = key;
      sessionActive = true;
      return true;
    },

    async startPracticeSession(cards: Card[], limit: number = 50, key: string | null = null) {
      reset();
      const selectedCards = shuffleCards(cards).slice(0, limit);
      if (selectedCards.length === 0) return false;
      dueCards = await toDueCards(selectedCards);
      sessionSize = dueCards.length;
      sessionKey = key;
      practiceMode = true;
      sessionActive = true;
      return true;
    },

    async startSession(deckIds: string[], limit: number = 50, key: string | null = null) {
      reset();
      currentDeckId = deckIds.length === 1 ? deckIds[0] : null;
      const cards = await api.getDueCards(deckIds, limit);
      if (cards.length === 0) return false;
      dueCards = cards;
      sessionSize = dueCards.length;
      sessionKey = key;
      currentIndex = 0;
      isFlipped = false;
      sessionActive = true;
      return true;
    },

    async startSessionByTags(tags: string[], limit: number = 50, key: string | null = null) {
      reset();
      currentDeckId = null;
      const cards = await api.getDueCardsByTags(tags, limit);
      if (cards.length === 0) return false;
      dueCards = cards;
      sessionSize = dueCards.length;
      sessionKey = key;
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

      const newState: CardState = practiceMode
        ? card.state
        : await api.submitReview(card.card.id, quality);

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
        } else {
          completedCount = Math.min(sessionSize, completedCount + 1);
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
        completedCount = Math.max(0, completedCount - 1);
      }
    },

    pauseSession() {
      isFlipped = false;
    },

    endSession() {
      reset();
    },
  };
}

const studyStore = getStudyStore();
export { studyStore };
