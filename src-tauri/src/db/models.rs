use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exam {
    pub id: String,
    pub name: String,
    pub exam_type: String,
    pub exam_date: String,
    pub created_at: String,
    pub deck_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamStats {
    pub total_cards: u32,
    pub mastered_cards: u32,
    pub cards_left: u32,
    pub days_left: i32,
    pub cards_per_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub deck_id: String,
    pub card_type: String,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub front_language: Option<String>,
    #[serde(default)]
    pub back_language: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub card_id: String,
    pub quality: u8,
    pub reviewed_at: String,
    pub interval: u32,
    pub ease_factor: f64,
    pub repetitions: u32,
    pub prev_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    pub card_id: String,
    pub interval: u32,
    pub ease_factor: f64,
    pub repetitions: u32,
    pub next_review: String,
    pub total_reviews: u32,
    pub correct_streak: u32,
    pub last_review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeckStats {
    pub total_cards: u32,
    pub due_cards: u32,
    pub new_cards: u32,
    pub learning_cards: u32,
    pub reviewing_cards: u32,
    pub mastered_cards: u32,
    pub avg_ease_factor: f64,
    pub avg_interval: f64,
    pub total_reviews_sum: u32,
    pub reviews_today: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    pub total_cards: u32,
    pub due_cards: u32,
    pub reviews_today: u32,
    pub avg_ease_factor: f64,
    pub streak_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub card: Card,
    pub deck_name: String,
}

impl CardState {
    pub fn new(card_id: &str) -> Self {
        Self {
            card_id: card_id.to_string(),
            interval: 0,
            ease_factor: 2.5,
            repetitions: 0,
            next_review: "1970-01-01".to_string(),
            total_reviews: 0,
            correct_streak: 0,
            last_review: None,
        }
    }
}

// ── Test Engine Models ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamTemplate {
    pub id: String,
    pub name: String,
    pub deck_ids: Vec<String>,
    pub tags: Vec<String>,
    pub allowed_card_types: Vec<String>,
    pub question_count: u32,
    pub time_limit_minutes: u32,
    pub pass_percentage: f64,
    pub seed: Option<u64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamSession {
    pub id: String,
    pub template_id: Option<String>,
    pub name: String,
    pub status: String, // "in_progress", "completed", "abandoned"
    pub started_at: String,
    pub finished_at: Option<String>,
    pub seed: u64,
    pub current_index: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamQuestion {
    pub id: String,
    pub session_id: String,
    pub card_id: String,
    pub question_index: u32,
    pub card_type: String,
    pub prompt: String,
    pub options_json: Option<String>,
    pub expected_answer: String,
    pub user_answer: Option<String>,
    pub is_correct: Option<bool>,
    pub points_earned: f64,
    pub max_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub key: String,
    pub total: u32,
    pub correct: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamResultBreakdown {
    pub by_deck: Vec<CategoryScore>,
    pub by_tag: Vec<CategoryScore>,
    pub by_card_type: Vec<CategoryScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamResult {
    pub session_id: String,
    pub score_percentage: f64,
    pub passed: bool,
    pub total_questions: u32,
    pub correct_count: u32,
    pub incorrect_count: u32,
    pub skipped_count: u32,
    pub duration_seconds: u64,
    pub breakdown: ExamResultBreakdown,
}
