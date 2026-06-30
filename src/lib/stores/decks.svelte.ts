import * as api from "$lib/api";
import type { Deck } from "$lib/types";

let decks = $state<Deck[]>([]);
let loading = $state(false);
let hasSeeded = $state(typeof localStorage !== 'undefined' && localStorage.getItem('stapelweise_seeded') === '1');

export function getDeckStore() {
  return {
    get decks() {
      return decks;
    },
    get loading() {
      return loading;
    },
    get hasSeeded() {
      return hasSeeded;
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
    async seed() {
      loading = true;
      try {
        const seeded = await api.seedSampleData();
        decks = [...decks, ...seeded];
        hasSeeded = true;
        localStorage.setItem('stapelweise_seeded', '1');
      } finally {
        loading = false;
      }
    },
  };
}

const deckStore = getDeckStore();
export { deckStore };
