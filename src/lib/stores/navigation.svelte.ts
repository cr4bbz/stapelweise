// Navigation & study-session store.
//
// Owns the top-level view ("decks" dashboard, single deck, study, search,
// settings, test) and the parameters of the active study/practice/test run.
// It also tracks elapsed study time and review counts, persisting a daily
// activity log used by the dashboard modules. Modal-driven actions (creating a
// deck or exam) stay in the page and are passed in as callbacks where needed.

import { deckStore } from "$lib/stores/decks.svelte";
import { dashboardStore } from "$lib/stores/dashboard.svelte";
import { t } from "$lib/i18n";
import type { Card, Deck } from "$lib/types";

export type AppView = "decks" | "cards" | "study" | "search" | "settings" | "test";

const lastDeckStorageKey = "stapelweise.learning.lastDeck.v1";
const activityStorageKey = "stapelweise.learning.activity.v1";

let view = $state<AppView>("decks");
let activeDeck = $state<Deck | null>(null);
let activeDeckIds = $state<string[]>([]);
let activeTags = $state<string[]>([]);
let activeCustomCards = $state<Card[]>([]);
let activeDeckName = $state("");
let activePracticeMode = $state(false);
let searchQuery = $state("");

let studyStartedAt: number | null = null;
let studyReviews = 0;

function resetActive() {
  activeDeck = null;
  activeDeckIds = [];
  activeTags = [];
  activeCustomCards = [];
  activePracticeMode = false;
  activeDeckName = "";
}

function beginStudyTracking() {
  studyStartedAt = Date.now();
  studyReviews = 0;
}

function finishStudyTracking() {
  if (view !== "study" || !studyStartedAt) return;
  const seconds = Math.max(1, Math.min(8 * 60 * 60, Math.round((Date.now() - studyStartedAt) / 1000)));
  const now = new Date();
  const key = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  try {
    const log = JSON.parse(localStorage.getItem(activityStorageKey) ?? "{}") as Record<string, { reviews: number; seconds: number }>;
    const current = log[key] ?? { reviews: 0, seconds: 0 };
    log[key] = { reviews: current.reviews + studyReviews, seconds: current.seconds + seconds };
    localStorage.setItem(activityStorageKey, JSON.stringify(log));
  } catch {
    // A full or unavailable localStorage should never break a study session.
  }
  studyStartedAt = null;
  studyReviews = 0;
}

// ── View switches ────────────────────────────────────────────

function openSearch() {
  searchQuery = "";
  view = "search";
}

function openSettings() {
  view = "settings";
}

/** Show the dashboard without the study-tracking side effects of goHome. */
function openDashboard() {
  resetActive();
  view = "decks";
}

function openDeck(deck: Deck) {
  activeDeck = deck;
  view = "cards";
}

function openDeckById(deckId: string) {
  const deck = deckStore.decks.find((candidate) => candidate.id === deckId);
  if (deck) openDeck(deck);
}

// ── Study runs ───────────────────────────────────────────────

function studyDeck(deck: Deck) {
  localStorage.setItem(lastDeckStorageKey, deck.id);
  beginStudyTracking();
  resetActive();
  activeDeck = deck;
  activeDeckIds = [deck.id];
  activeDeckName = deck.name;
  view = "study";
}

function studyDecks(decks: Deck[]) {
  if (decks.length === 1) localStorage.setItem(lastDeckStorageKey, decks[0].id);
  beginStudyTracking();
  resetActive();
  activeDeck = decks.length === 1 ? decks[0] : null;
  activeDeckIds = decks.map((deck) => deck.id);
  activeDeckName = decks.length === 1 ? decks[0].name : `${decks.length} ${t("Stapel")}`;
  view = "study";
}

function studyToday() {
  const stats = dashboardStore.stats;
  if (!stats || stats.due_cards === 0 || deckStore.decks.length === 0) return;
  beginStudyTracking();
  resetActive();
  activeDeckIds = deckStore.decks.map((deck) => deck.id);
  activeDeckName = t("todayLearn");
  view = "study";
}

function practiceDeck(deck: Deck) {
  localStorage.setItem(lastDeckStorageKey, deck.id);
  beginStudyTracking();
  resetActive();
  activeDeck = deck;
  activeDeckIds = [deck.id];
  activePracticeMode = true;
  activeDeckName = `${deck.name} · ${t("freePractice")}`;
  view = "study";
}

function practiceAllDecks() {
  if (deckStore.decks.length === 0) return;
  beginStudyTracking();
  resetActive();
  activeDeckIds = deckStore.decks.map((deck) => deck.id);
  activePracticeMode = true;
  activeDeckName = t("freePractice");
  view = "study";
}

function studyTags(tags: string[]) {
  beginStudyTracking();
  resetActive();
  activeDeckIds = deckStore.decks.map((deck) => deck.id);
  activeTags = tags;
  activeDeckName = tags.length === 1 ? `#${tags[0]}` : `${tags.length} ${t("Tags")}`;
  view = "study";
}

function studyExam(deckIds: string[], examName: string) {
  beginStudyTracking();
  resetActive();
  activeDeckIds = deckIds;
  activeDeckName = `${t("Prüfung")}: ${examName}`;
  view = "study";
}

function simulateExam(deckIds: string[], examName: string) {
  resetActive();
  activeDeckIds = deckIds;
  activeDeckName = `${t("Simulation")}: ${examName}`;
  view = "test";
}

function studyCards(cards: Card[], name: string) {
  if (cards.length === 0) return;
  beginStudyTracking();
  resetActive();
  activeCustomCards = cards;
  activeDeckName = name;
  view = "study";
}

/** Study the cards that were answered incorrectly in a test run. */
function studyTestFailures(cards: Card[]) {
  resetActive();
  activeCustomCards = cards;
  activeDeckName = `${cards.length} ${t("Falsche Testkarten")}`;
  view = "study";
}

/**
 * The context-aware primary action: create a deck, add cards, learn what's due,
 * or free-practice. Creating a deck opens a modal, so that path is delegated.
 */
function primaryAction(onRequestNewDeck: () => void) {
  const stats = dashboardStore.stats;
  if (deckStore.decks.length === 0) {
    onRequestNewDeck();
    return;
  }
  if (stats && stats.total_cards === 0) {
    openDeck(deckStore.decks[0]);
    return;
  }
  if (stats && stats.due_cards > 0) {
    studyToday();
    return;
  }
  practiceAllDecks();
}

// ── Returning to the dashboard ───────────────────────────────

function goHome() {
  finishStudyTracking();
  dashboardStore.invalidate();
  view = "decks";
  resetActive();
}

function closeStudy() {
  finishStudyTracking();
  dashboardStore.invalidate();
  if (activeDeck) {
    view = "cards";
    activeDeckIds = [];
    activeTags = [];
    activeCustomCards = [];
    activePracticeMode = false;
    activeDeckName = "";
    return;
  }
  goHome();
}

function recordReview() {
  studyReviews += 1;
}

export function getNavigationStore() {
  return {
    get view() {
      return view;
    },
    get activeDeck() {
      return activeDeck;
    },
    get activeDeckIds() {
      return activeDeckIds;
    },
    get activeTags() {
      return activeTags;
    },
    get activeCustomCards() {
      return activeCustomCards;
    },
    get activeDeckName() {
      return activeDeckName;
    },
    get activePracticeMode() {
      return activePracticeMode;
    },
    get searchQuery() {
      return searchQuery;
    },
    get hasDecks() {
      return deckStore.decks.length > 0;
    },
    /** Whether the study view has something to render. */
    get hasActiveStudy() {
      return activeDeckIds.length > 0 || activeTags.length > 0 || activeCustomCards.length > 0;
    },
    /** Label for the focus module's primary button, based on current state. */
    get primaryActionLabel() {
      const stats = dashboardStore.stats;
      if (deckStore.decks.length === 0) return t("Ersten Stapel anlegen");
      if (stats && stats.total_cards === 0) return t("Karten anlegen");
      if (stats && stats.due_cards > 0) return t("startSession");
      return t("practice");
    },
    /** Human phrase describing today's learning load. */
    get learningLoad() {
      const stats = dashboardStore.stats;
      if (!stats) return t("Bereit");
      if (deckStore.decks.length === 0) return t("Startklar");
      if (stats.due_cards === 0) return t("Alles im Rhythmus");
      if (stats.due_cards <= 12) return t("Kleine Runde");
      if (stats.due_cards <= 35) return t("Guter Fokusblock");
      return t("Aufholsession");
    },
    openSearch,
    openSettings,
    openDashboard,
    openDeck,
    openDeckById,
    studyDeck,
    studyDecks,
    studyToday,
    practiceDeck,
    practiceAllDecks,
    studyTags,
    studyExam,
    simulateExam,
    studyCards,
    studyTestFailures,
    primaryAction,
    goHome,
    closeStudy,
    recordReview,
  };
}

const navigation = getNavigationStore();
export { navigation };
