import { invoke } from "@tauri-apps/api/core";
import type { Card, CardState, Deck, DueCard } from "./types";

// ── Decks ────────────────────────────────────────────

export async function createDeck(name: string): Promise<Deck> {
  return invoke("create_deck", { name });
}

export async function listDecks(): Promise<Deck[]> {
  return invoke("list_decks");
}

export async function getDeck(deckId: string): Promise<Deck | null> {
  return invoke("get_deck", { deckId });
}

export async function updateDeck(deckId: string, name: string): Promise<void> {
  return invoke("update_deck", { deckId, name });
}

export async function deleteDeck(deckId: string): Promise<void> {
  return invoke("delete_deck", { deckId });
}

// ── Cards ────────────────────────────────────────────

export async function createCard(
  deckId: string,
  front: string,
  back: string
): Promise<Card> {
  return invoke("create_card", { deckId, front, back });
}

export async function listCards(deckId: string): Promise<Card[]> {
  return invoke("list_cards", { deckId });
}

export async function updateCard(
  cardId: string,
  front: string,
  back: string
): Promise<void> {
  return invoke("update_card", { cardId, front, back });
}

export async function deleteCard(cardId: string): Promise<void> {
  return invoke("delete_card", { cardId });
}

// ── Study ────────────────────────────────────────────

export async function getDueCards(
  deckId: string,
  limit: number
): Promise<DueCard[]> {
  return invoke("get_due_cards", { deckId, limit });
}

export async function countDueCards(deckId: string): Promise<number> {
  return invoke("count_due_cards", { deckId });
}

export async function countTotalCards(deckId: string): Promise<number> {
  return invoke("count_total_cards", { deckId });
}

export async function submitReview(
  cardId: string,
  quality: number
): Promise<CardState> {
  return invoke("submit_review", { cardId, quality });
}
