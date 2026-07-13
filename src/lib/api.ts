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

export async function importDeckFromJson(name: string, cards: any[]): Promise<void> {
  return cmd("import_deck", { name, cards });
}

export async function seedSampleData(): Promise<Deck[]> {
  return cmd("seed_sample_data");
}

export async function syncObsidianVault(vaultPath: string, deckName: string): Promise<Deck> {
  return cmd("sync_obsidian_vault", { vaultPath, deckName });
}

// ── Cards ────────────────────────────────────────────

export async function createCard(
  deckId: string,
  front: string,
  back: string,
  reasoning: string | null = null,
  cardType: string = "basic",
  content: string | null = null,
  tags: string[] = []
): Promise<Card> {
  return cmd("create_card", { deckId, front, back, reasoning, cardType, content, tags });
}

export async function listCards(deckId: string): Promise<Card[]> {
  return cmd("list_cards", { deckId });
}

export async function updateCard(
  cardId: string,
  front: string,
  back: string,
  reasoning: string | null = null,
  cardType: string = "basic",
  content: string | null = null,
  tags: string[] = []
): Promise<void> {
  return cmd("update_card", { cardId, front, back, reasoning, cardType, content, tags });
}

export async function deleteCard(cardId: string): Promise<void> {
  return cmd("delete_card", { cardId });
}

export async function getCardState(cardId: string): Promise<CardState | null> {
  return cmd("get_card_state", { cardId });
}

export async function getAllTags(): Promise<string[]> {
  return cmd("get_all_tags");
}

// ── Study ────────────────────────────────────────────

export async function getDueCards(
  deckIds: string[],
  limit: number
): Promise<DueCard[]> {
  return cmd("get_due_cards", { deckIds, limit });
}

export async function getDueCardsByTags(
  tags: string[],
  limit: number
): Promise<DueCard[]> {
  return cmd("get_due_cards_by_tags", { tags, limit });
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

// ── EXAMS ──────────────────────────────────────────────

export async function createExam(
  name: string,
  examType: string,
  examDate: string,
  deckIds: string[]
): Promise<any> {
  return cmd("create_exam", { name, examType, examDate, deckIds });
}

export async function listExams(): Promise<any[]> {
  return cmd("list_exams");
}

export async function deleteExam(id: string): Promise<void> {
  return cmd("delete_exam", { id });
}

export async function updateExam(
  id: string,
  name: string,
  examType: string,
  examDate: string,
  deckIds: string[]
): Promise<any> {
  return cmd("update_exam", { id, name, examType, examDate, deckIds });
}

export async function getExamStats(id: string): Promise<any> {
  return cmd("get_exam_stats", { id });
}
