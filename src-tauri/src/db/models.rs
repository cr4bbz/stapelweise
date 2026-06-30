use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
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
