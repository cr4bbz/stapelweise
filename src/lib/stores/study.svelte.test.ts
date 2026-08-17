import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("$lib/api", () => ({
  getCardState: vi.fn(),
  getDueCards: vi.fn(),
  getDueCardsByTags: vi.fn(),
  submitReview: vi.fn(),
  undoLastReview: vi.fn(),
  countDueCards: vi.fn(),
  countTotalCards: vi.fn(),
}));

import * as api from "$lib/api";
import type { Card, CardState, DueCard } from "$lib/types";
import { settingsStore } from "./settings.svelte";
import { getStudyStore } from "./study.svelte";

const baseState: CardState = {
  card_id: "c1",
  interval: 0,
  ease_factor: 2.5,
  repetitions: 0,
  next_review: "2026-08-17",
  total_reviews: 0,
  correct_streak: 0,
  last_review: null,
};

function card(id: string, deck = "d1"): Card {
  return {
    id,
    deck_id: deck,
    card_type: "basic",
    content: null,
    reasoning: null,
    front: `Front ${id}`,
    back: `Back ${id}`,
    front_language: null,
    back_language: null,
    tags: [],
    created_at: "2026-08-17",
    updated_at: "2026-08-17",
  } as Card;
}

function due(id: string): DueCard {
  return { card: card(id), state: { ...baseState, card_id: id } };
}

beforeEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
  settingsStore.current.sm2_pass_threshold = 3;
});

it("does not start an empty custom session", async () => {
  const store = getStudyStore();
  expect(await store.startCustomSession([])).toBe(false);
  expect(store.sessionActive).toBe(false);
});

it("starts a due-card session", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1"), due("c2")]);
  const store = getStudyStore();
  expect(await store.startSession(["d1"], 50, "deck:d1")).toBe(true);
  expect(store.sessionSize).toBe(2);
  expect(store.currentCard?.card.id).toBe("c1");
});

it("starts a tag session", async () => {
  vi.mocked(api.getDueCardsByTags).mockResolvedValue([due("c1")]);
  const store = getStudyStore();
  expect(await store.startSessionByTags(["logic"], 20, "tag:logic")).toBe(true);
  expect(api.getDueCardsByTags).toHaveBeenCalledWith(["logic"], 20);
});

it("persists resumable sessions", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1")]);
  const first = getStudyStore();
  await first.startSession(["d1"], 10, "resume-key");
  first.pauseSession();
  const second = getStudyStore();
  second.endSession();
  localStorage.setItem(
    "stapelweise.study-session.v1",
    JSON.stringify({
      version: 1,
      dueCards: [due("c1")],
      currentIndex: 0,
      practiceMode: false,
      sessionSize: 1,
      completedCount: 0,
      sessionKey: "resume-key",
    })
  );
  expect(second.resumeSession("resume-key")).toBe(true);
  expect(second.currentCard?.card.id).toBe("c1");
});

it("flip only reveals the current card", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1")]);
  const store = getStudyStore();
  await store.startSession(["d1"]);
  store.flip();
  expect(store.isFlipped).toBe(true);
});

it("passing a card completes it", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1")]);
  vi.mocked(api.submitReview).mockResolvedValue({ ...baseState, repetitions: 1 });
  const store = getStudyStore();
  await store.startSession(["d1"]);
  await store.rate(4);
  expect(store.completedCount).toBe(1);
  expect(store.sessionActive).toBe(false);
});

it("failed cards return to the end of the round", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1"), due("c2")]);
  vi.mocked(api.submitReview).mockResolvedValue({ ...baseState, interval: 1 });
  const store = getStudyStore();
  await store.startSession(["d1"]);
  await store.rate(2);
  expect(store.dueCards.map((item) => item.card.id)).toEqual(["c2", "c1"]);
  expect(store.completedCount).toBe(0);
});

it("practice sessions never submit reviews", async () => {
  vi.mocked(api.getCardState).mockResolvedValue(baseState);
  const store = getStudyStore();
  await store.startPracticeSession([card("c1")], 10, "practice");
  await store.rate(4);
  expect(api.submitReview).not.toHaveBeenCalled();
});

it("practice undo is local", async () => {
  vi.mocked(api.getCardState).mockResolvedValue(baseState);
  const store = getStudyStore();
  await store.startPracticeSession([card("c1")], 10, "practice");
  await store.rate(4);
  expect(await store.undo()).toBe(true);
  expect(api.undoLastReview).not.toHaveBeenCalled();
  expect(store.currentCard?.card.id).toBe("c1");
});

it("normal undo asks backend for the deck-scoped review", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1")]);
  vi.mocked(api.submitReview).mockResolvedValue(baseState);
  vi.mocked(api.undoLastReview).mockResolvedValue(due("c1"));
  const store = getStudyStore();
  await store.startSession(["d1"]);
  await store.rate(4);
  expect(await store.undo()).toBe(true);
  expect(api.undoLastReview).toHaveBeenCalledWith("d1");
});

it("loads due and total counts together", async () => {
  vi.mocked(api.countDueCards).mockResolvedValue(3);
  vi.mocked(api.countTotalCards).mockResolvedValue(12);
  const store = getStudyStore();
  await store.loadCounts("d1");
  expect(store.dueCount).toBe(3);
  expect(store.totalCount).toBe(12);
});

it("endSession clears persisted state", async () => {
  vi.mocked(api.getDueCards).mockResolvedValue([due("c1")]);
  const store = getStudyStore();
  await store.startSession(["d1"], 10, "key");
  expect(localStorage.getItem("stapelweise.study-session.v1")).not.toBeNull();
  store.endSession();
  expect(localStorage.getItem("stapelweise.study-session.v1")).toBeNull();
  expect(store.sessionActive).toBe(false);
});
