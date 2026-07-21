import * as api from "$lib/api";
import type { Deck } from "$lib/types";

let decks = $state<Deck[]>([]);
let loading = $state(false);

export function getDeckStore() {
  return {
    get decks() {
      return decks;
    },
    get loading() {
      return loading;
    },
    async load() {
      loading = true;
      try {
        decks = await api.listDecks();
      } finally {
        loading = false;
      }
    },
    async create(name: string): Promise<Deck> {
      const deck = await api.createDeck(name);
      decks = [deck, ...decks];
      return deck;
    },
    async update(deckId: string, name: string) {
      await api.updateDeck(deckId, name);
      const idx = decks.findIndex((d) => d.id === deckId);
      if (idx !== -1) {
        decks[idx] = { ...decks[idx], name };
      }
    },
    async remove(deckId: string) {
      await api.deleteDeck(deckId);
      decks = decks.filter((d) => d.id !== deckId);
    },
    async archive(deckId: string) {
      await api.archiveDeck(deckId);
      decks = decks.filter((d) => d.id !== deckId);
    },
    async restore(deckId: string): Promise<Deck> {
      await api.restoreDeck(deckId);
      const deck = await api.getDeck(deckId);
      if (!deck) throw new Error("Deck not found after restore");
      decks = [deck, ...decks];
      return deck;
    },
    async seed() {
      loading = true;
      try {
        const seeded = await api.seedSampleData();
        decks = [...decks, ...seeded];
      } finally {
        loading = false;
      }
    },
  };
}

const deckStore = getDeckStore();
export { deckStore };
