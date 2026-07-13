import { invoke } from "@tauri-apps/api/core";
import {
  StapelweiseError,
  type AppSettings,
  type Card,
  type CardState,
  type CardType,
  type DashboardStats,
  type Deck,
  type DeckStats,
  type DueCard,
  type Exam,
  type ExamStats,
  type JsonCardInput,
  type SearchResult,
  type SerializedAppError,
} from "./types";

function isSerializedAppError(obj: unknown): obj is SerializedAppError {
  return (
    typeof obj === "object" &&
    obj !== null &&
    "code" in obj &&
    "message" in obj &&
    typeof (obj as SerializedAppError).code === "string" &&
    typeof (obj as SerializedAppError).message === "string"
  );
}

// Helper to log and rethrow structured StapelweiseError
async function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (e: unknown) {
    if (isSerializedAppError(e)) {
      const err = new StapelweiseError(e);
      console.error(`[stapelweise] command "${name}" failed [${err.code}]:`, err.message, err.details || "");
      throw err;
    }
    let fallbackStr = "Command failed";
    try {
      fallbackStr = typeof e === "string" ? e : JSON.stringify(e);
    } catch {
      fallbackStr = String(e);
    }
    const fallbackErr = new StapelweiseError({
      code: "INTERNAL_ERROR",
      message: fallbackStr || `Command ${name} failed`,
      details: null,
      recoverable: false,
    });
    console.error(`[stapelweise] command "${name}" failed:`, fallbackStr);
    throw fallbackErr;
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

export async function importDeckFromJson(name: string, cards: JsonCardInput[]): Promise<void> {
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
  cardType: CardType = "basic",
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
  cardType: CardType = "basic",
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
): Promise<Exam> {
  return cmd("create_exam", { name, examType, examDate, deckIds });
}

export async function listExams(): Promise<Exam[]> {
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
): Promise<Exam> {
  return cmd("update_exam", { id, name, examType, examDate, deckIds });
}

export async function getExamStats(id: string): Promise<ExamStats> {
  return cmd("get_exam_stats", { id });
}
