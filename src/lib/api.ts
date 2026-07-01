import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, Card, CardState, DashboardStats, Deck, DeckStats, DueCard, SearchResult } from "./types";

// Helper to log and rethrow
async function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (e) {
    console.error(`[stapelweise] command "${name}" failed:`, e);
    throw e;
  }
}

// ── Decks ────────────────────────────────────────────

export async function createDeck(name: string): Promise<Deck> {
  return cmd("create_deck", { name });
}

export async function listDecks(): Promise<Deck[]> {
  return cmd("list_decks");
}

export async function getDeck(deckId: string): Promise<Deck | null> {
  return cmd("get_deck", { deckId });
}

export async function updateDeck(deckId: string, name: string): Promise<void> {
  return cmd("update_deck", { deckId, name });
}

export async function deleteDeck(deckId: string): Promise<void> {
  return cmd("delete_deck", { deckId });
}

export async function seedSampleData(): Promise<Deck[]> {
  return cmd("seed_sample_data");
}

// ── Cards ────────────────────────────────────────────

export async function createCard(
  deckId: string,
  front: string,
  back: string
): Promise<Card> {
  return cmd("create_card", { deckId, front, back });
}

export async function listCards(deckId: string): Promise<Card[]> {
  return cmd("list_cards", { deckId });
}

export async function updateCard(
  cardId: string,
  front: string,
  back: string
): Promise<void> {
  return cmd("update_card", { cardId, front, back });
}

export async function deleteCard(cardId: string): Promise<void> {
  return cmd("delete_card", { cardId });
}

export async function getCardState(cardId: string): Promise<CardState | null> {
  return cmd("get_card_state", { cardId });
}

// ── Study ────────────────────────────────────────────

export async function getDueCards(
  deckIds: string[],
  limit: number
): Promise<DueCard[]> {
  return cmd("get_due_cards", { deckIds, limit });
}

export async function countDueCards(deckId: string): Promise<number> {
  return cmd("count_due_cards", { deckId });
}

export async function countTotalCards(deckId: string): Promise<number> {
  return cmd("count_total_cards", { deckId });
}

export async function submitReview(
  cardId: string,
  quality: number
): Promise<CardState> {
  return cmd("submit_review", { cardId, quality });
}

export async function undoLastReview(deckId: string): Promise<DueCard | null> {
  return cmd("undo_last_review", { deckId });
}

// ── Search ────────────────────────────────────────────

export async function searchCards(query: string): Promise<SearchResult[]> {
  return cmd("search_cards", { query });
}

// ── Settings ──────────────────────────────────────────

export async function getSettings(): Promise<AppSettings> {
  return cmd("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return cmd("update_settings", { settings });
}

// ── Stats ──────────────────────────────────────────────

export async function getDeckStats(deckId: string): Promise<DeckStats> {
  return cmd("get_deck_stats", { deckId });
}

export async function getDashboardStats(): Promise<DashboardStats> {
  return cmd("get_dashboard_stats");
}
