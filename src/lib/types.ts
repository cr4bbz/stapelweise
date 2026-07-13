// Mirrors Rust types from src-tauri/src/db/models.rs and error definitions

export type CardType = "basic" | "cloze" | "multiple_choice" | "free_text" | "ordering";

export interface MultipleChoiceOption {
  text: string;
  correct: boolean;
}

export interface MultipleChoiceContent {
  options: MultipleChoiceOption[];
}

export interface OrderingContent {
  items: string[];
}

export interface BaseCard {
  id: string;
  deck_id: string;
  card_type: CardType;
  content: string | null;
  reasoning: string | null;
  front: string;
  back: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface BasicCard extends BaseCard {
  card_type: "basic";
}

export interface ClozeCard extends BaseCard {
  card_type: "cloze";
}

export interface MultipleChoiceCard extends BaseCard {
  card_type: "multiple_choice";
}

export interface FreeTextCard extends BaseCard {
  card_type: "free_text";
}

export interface OrderingCard extends BaseCard {
  card_type: "ordering";
}

export type Card = BasicCard | ClozeCard | MultipleChoiceCard | FreeTextCard | OrderingCard;

export interface JsonCardInput {
  front: string;
  back: string;
  card_type?: CardType;
  content?: string | null;
  reasoning?: string | null;
  tags?: string[];
}

export interface Deck {
  id: string;
  name: string;
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
  obsidian_vault_path: string;
  obsidian_flashcard_tag: string;
}

export interface Exam {
  id: string;
  name: string;
  exam_type: string;
  exam_date: string;
  created_at: string;
  deck_ids: string[];
}

export interface ExamStats {
  total_cards: number;
  mastered_cards: number;
  cards_left: number;
  days_left: number;
  cards_per_day: number;
}

// ── Test Engine Types ──────────────────────────────────────

export interface ExamTemplate {
  id: string;
  name: string;
  deck_ids: string[];
  tags: string[];
  allowed_card_types: CardType[];
  question_count: number;
  time_limit_minutes: number;
  pass_percentage: number;
  seed: number | null;
  created_at: string;
}

export interface ExamSession {
  id: string;
  template_id: string | null;
  name: string;
  status: "in_progress" | "completed" | "abandoned";
  started_at: string;
  finished_at: string | null;
  seed: number;
  current_index: number;
  created_at: string;
}

export interface ExamQuestion {
  id: string;
  session_id: string;
  card_id: string;
  question_index: number;
  card_type: CardType;
  prompt: string;
  options_json: string | null;
  expected_answer: string;
  user_answer: string | null;
  is_correct: boolean | null;
  points_earned: number;
  max_points: number;
}

export interface CategoryScore {
  key: string;
  total: number;
  correct: number;
  percentage: number;
}

export interface ExamResultBreakdown {
  by_deck: CategoryScore[];
  by_tag: CategoryScore[];
  by_card_type: CategoryScore[];
}

export interface ExamResult {
  session_id: string;
  score_percentage: number;
  passed: boolean;
  total_questions: number;
  correct_count: number;
  incorrect_count: number;
  skipped_count: number;
  duration_seconds: number;
  breakdown: ExamResultBreakdown;
}

// Structured error payload from Rust backend
export type ErrorCode =
  | "VALIDATION_FAILED"
  | "NOT_FOUND"
  | "DB_ERROR"
  | "IMPORT_INVALID_JSON"
  | "IMPORT_FAILED"
  | "EXPORT_FAILED"
  | "OBSIDIAN_SYNC_ERROR"
  | "INTERNAL_ERROR";

export interface SerializedAppError {
  code: ErrorCode;
  message: string;
  details?: string | null;
  recoverable: boolean;
}

export class StapelweiseError extends Error {
  code: ErrorCode;
  details?: string | null;
  recoverable: boolean;

  constructor(payload: SerializedAppError) {
    super(payload.message);
    this.name = "StapelweiseError";
    this.code = payload.code;
    this.details = payload.details;
    this.recoverable = payload.recoverable;
  }
}
