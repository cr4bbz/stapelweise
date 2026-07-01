// Mirrors Rust types from src-tauri/src/db/models.rs

export interface Deck {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface Card {
  id: string;
  deck_id: string;
  front: string;
  back: string;
  created_at: string;
  updated_at: string;
}

export interface CardState {
  card_id: string;
  interval: number;
  ease_factor: number;
  repetitions: number;
  next_review: string;
  total_reviews: number;
  correct_streak: number;
  last_review: string | null;
}

export interface DueCard {
  card: Card;
  state: CardState;
}

export interface DeckStats {
  total_cards: number;
  due_cards: number;
  new_cards: number;
  learning_cards: number;
  reviewing_cards: number;
  mastered_cards: number;
  avg_ease_factor: number;
  avg_interval: number;
  total_reviews_sum: number;
  reviews_today: number;
}

export interface SearchResult {
  card: Card;
  deck_name: string;
}

export interface DashboardStats {
  total_cards: number;
  due_cards: number;
  reviews_today: number;
  avg_ease_factor: number;
  streak_days: number;
}

export interface AppSettings {
  theme: "light" | "dark" | "auto";
  card_font_family: "serif" | "sans";
  card_font_size: "small" | "medium" | "large";
  session_limit: number;
  sm2_initial_ef: number;
  sm2_pass_threshold: number;
}
