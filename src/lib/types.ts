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
