use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};

use super::models::{Card, CardState, DashboardStats, Deck, DeckStats, Review, SearchResult};

pub struct Repository {
    conn: Connection,
}

impl Repository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    // ── Decks ──────────────────────────────────────────

    pub fn create_deck(&self, name: &str) -> Result<Deck> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO decks (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, now, now],
        )?;

        Ok(Deck {
            id,
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_decks(&self) -> Result<Vec<Deck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at, updated_at FROM decks ORDER BY created_at DESC")?;

        let decks = stmt
            .query_map([], |row| {
                Ok(Deck {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(decks)
    }

    pub fn get_deck(&self, deck_id: &str) -> Result<Option<Deck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, created_at, updated_at FROM decks WHERE id = ?1")?;

        let mut rows = stmt.query_map(params![deck_id], |row| {
            Ok(Deck {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;

        match rows.next() {
            Some(deck) => Ok(Some(deck?)),
            None => Ok(None),
        }
    }

    pub fn update_deck(&self, deck_id: &str, name: &str) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.conn.execute(
            "UPDATE decks SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, deck_id],
        )?;
        Ok(())
    }

    pub fn delete_deck(&self, deck_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM decks WHERE id = ?1", params![deck_id])?;
        Ok(())
    }

    pub fn get_or_create_deck(&self, name: &str) -> Result<Deck> {
        let mut stmt = self.conn.prepare("SELECT id, name, created_at, updated_at FROM decks WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(Deck {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        if let Some(deck) = rows.next() {
            return deck;
        }
        self.create_deck(name)
    }

    // ── Obsidian ───────────────────────────────────────

    pub fn get_obsidian_card_hash(&self, file_path: &str) -> Result<Option<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT card_id, hash FROM obsidian_cards WHERE file_path = ?1")?;
        let mut rows = stmt.query_map(params![file_path], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn upsert_obsidian_card(&self, card_id: &str, file_path: &str, line_number: u32, hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO obsidian_cards (card_id, file_path, line_number, hash) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(card_id) DO UPDATE SET file_path = excluded.file_path, line_number = excluded.line_number, hash = excluded.hash",
            params![card_id, file_path, line_number, hash],
        )?;
        Ok(())
    }

    pub fn get_all_obsidian_file_paths(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT file_path FROM obsidian_cards")?;
        let paths = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>>>()?;
        Ok(paths)
    }

    // ── Cards ──────────────────────────────────────────

    pub fn set_card_tags(&self, card_id: &str, tags: &[String]) -> Result<()> {
        self.conn.execute("DELETE FROM card_tags WHERE card_id = ?1", params![card_id])?;
        for tag in tags {
            let tag_id = Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
                params![tag_id, tag],
            )?;
            let existing_tag_id: String = self.conn.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![tag],
                |row| row.get(0),
            )?;
            self.conn.execute(
                "INSERT INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
                params![card_id, existing_tag_id],
            )?;
        }
        Ok(())
    }

    pub fn get_all_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM tags ORDER BY name")?;
        let tags = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>>>()?;
        Ok(tags)
    }

    pub fn create_card(&self, deck_id: &str, card_type: &str, content: Option<&str>, reasoning: Option<&str>, front: &str, back: &str, tags: Vec<String>) -> Result<Card> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO cards (id, deck_id, card_type, content, reasoning, front, back, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, deck_id, card_type, content, reasoning, front, back, now, now],
        )?;

        // Create initial card_state for this card
        self.conn.execute(
            "INSERT INTO card_state (card_id) VALUES (?1)",
            params![id],
        )?;

        self.set_card_tags(&id, &tags)?;

        Ok(Card {
            id,
            deck_id: deck_id.to_string(),
            card_type: card_type.to_string(),
            content: content.map(|s| s.to_string()),
            reasoning: reasoning.map(|s| s.to_string()),
            front: front.to_string(),
            back: back.to_string(),
            tags,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Insert a card with a specific card_state, card_type, content, reasoning (for seed data / testing).
    pub fn seed_insert_card_with_state(
        &self,
        deck_id: &str,
        front: &str,
        back: &str,
        tags: Vec<String>,
        state: &CardState,
        card_type: &str,
        content: Option<&str>,
        reasoning: Option<&str>,
    ) -> Result<Card> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO cards (id, deck_id, card_type, content, reasoning, front, back, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, deck_id, card_type, content, reasoning, front, back, now, now],
        )?;

        self.conn.execute(
            "INSERT INTO card_state (card_id, interval, ease_factor, repetitions, next_review, total_reviews, correct_streak, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                state.interval,
                state.ease_factor,
                state.repetitions,
                state.next_review,
                state.total_reviews,
                state.correct_streak,
                state.last_review,
            ],
        )?;

        self.set_card_tags(&id, &tags)?;

        Ok(Card {
            id,
            deck_id: deck_id.to_string(),
            card_type: card_type.to_string(),
            content: content.map(|s| s.to_string()),
            reasoning: reasoning.map(|s| s.to_string()),
            front: front.to_string(),
            back: back.to_string(),
            tags,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_cards(&self, deck_id: &str) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at, GROUP_CONCAT(t.name) 
             FROM cards c
             LEFT JOIN card_tags ct ON ct.card_id = c.id
             LEFT JOIN tags t ON t.id = ct.tag_id
             WHERE c.deck_id = ?1
             GROUP BY c.id
             ORDER BY c.created_at DESC",
        )?;

        let cards = stmt
            .query_map(params![deck_id], |row| {
                let tags_str: Option<String> = row.get(9)?;
                let tags = tags_str
                    .map(|s| s.split(',').map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                Ok(Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    card_type: row.get(2)?,
                    content: row.get(3)?,
                    reasoning: row.get(4)?,
                    front: row.get(5)?,
                    back: row.get(6)?,
                    tags,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(cards)
    }

    pub fn list_all_cards(&self) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at, GROUP_CONCAT(t.name) 
             FROM cards c
             LEFT JOIN card_tags ct ON ct.card_id = c.id
             LEFT JOIN tags t ON t.id = ct.tag_id
             GROUP BY c.id
             ORDER BY c.created_at DESC",
        )?;

        let cards = stmt
            .query_map([], |row| {
                let tags_str: Option<String> = row.get(9)?;
                let tags = tags_str
                    .map(|s| s.split(',').map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                Ok(Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    card_type: row.get(2)?,
                    content: row.get(3)?,
                    reasoning: row.get(4)?,
                    front: row.get(5)?,
                    back: row.get(6)?,
                    tags,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(cards)
    }

    pub fn get_card(&self, card_id: &str) -> Result<Option<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at, GROUP_CONCAT(t.name)
             FROM cards c
             LEFT JOIN card_tags ct ON ct.card_id = c.id
             LEFT JOIN tags t ON t.id = ct.tag_id
             WHERE c.id = ?1
             GROUP BY c.id",
        )?;

        let mut rows = stmt.query_map(params![card_id], |row| {
            let tags_str: Option<String> = row.get(9)?;
            let tags = tags_str
                .map(|s| s.split(',').map(|t| t.to_string()).collect())
                .unwrap_or_default();

            Ok(Card {
                id: row.get(0)?,
                deck_id: row.get(1)?,
                card_type: row.get(2)?,
                content: row.get(3)?,
                reasoning: row.get(4)?,
                front: row.get(5)?,
                back: row.get(6)?,
                tags,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        match rows.next() {
            Some(card) => Ok(Some(card?)),
            None => Ok(None),
        }
    }

    pub fn update_card(&self, card_id: &str, card_type: &str, content: Option<&str>, reasoning: Option<&str>, front: &str, back: &str, tags: Vec<String>) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.conn.execute(
            "UPDATE cards SET card_type = ?1, content = ?2, reasoning = ?3, front = ?4, back = ?5, updated_at = ?6 WHERE id = ?7",
            params![card_type, content, reasoning, front, back, now, card_id],
        )?;
        
        self.set_card_tags(card_id, &tags)?;
        
        Ok(())
    }

    pub fn delete_card(&self, card_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM cards WHERE id = ?1", params![card_id])?;
        Ok(())
    }

    // ── Card State ─────────────────────────────────────

    pub fn get_card_state(&self, card_id: &str) -> Result<Option<CardState>> {
        let mut stmt = self.conn.prepare(
            "SELECT card_id, interval, ease_factor, repetitions, next_review, total_reviews, correct_streak, last_review FROM card_state WHERE card_id = ?1",
        )?;

        let mut rows = stmt.query_map(params![card_id], |row| {
            Ok(CardState {
                card_id: row.get(0)?,
                interval: row.get(1)?,
                ease_factor: row.get(2)?,
                repetitions: row.get(3)?,
                next_review: row.get(4)?,
                total_reviews: row.get(5)?,
                correct_streak: row.get(6)?,
                last_review: row.get(7)?,
            })
        })?;

        match rows.next() {
            Some(state) => Ok(Some(state?)),
            None => Ok(None),
        }
    }

    pub fn update_card_state(&self, state: &CardState) -> Result<()> {
        self.conn.execute(
            "UPDATE card_state SET interval = ?1, ease_factor = ?2, repetitions = ?3, next_review = ?4, total_reviews = ?5, correct_streak = ?6, last_review = ?7 WHERE card_id = ?8",
            params![
                state.interval,
                state.ease_factor,
                state.repetitions,
                state.next_review,
                state.total_reviews,
                state.correct_streak,
                state.last_review,
                state.card_id,
            ],
        )?;
        Ok(())
    }

    // ── Reviews ────────────────────────────────────────

    pub fn insert_review(&self, review: &Review) -> Result<()> {
        self.conn.execute(
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions, prev_state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                review.id,
                review.card_id,
                review.quality,
                review.reviewed_at,
                review.interval,
                review.ease_factor,
                review.repetitions,
                review.prev_state,
            ],
        )?;
        Ok(())
    }

    /// Atomically insert a review and update the corresponding card_state.
    /// Both operations happen in a single transaction — if either fails,
    /// neither is persisted.
    pub fn apply_review(&self, review: &Review, updated_state: &CardState) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;

        let result = self.conn.execute(
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions, prev_state) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                review.id,
                review.card_id,
                review.quality,
                review.reviewed_at,
                review.interval,
                review.ease_factor,
                review.repetitions,
                review.prev_state,
            ],
        );

        if let Err(e) = result {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        let result = self.conn.execute(
            "UPDATE card_state SET interval = ?1, ease_factor = ?2, repetitions = ?3, next_review = ?4, total_reviews = ?5, correct_streak = ?6, last_review = ?7 WHERE card_id = ?8",
            params![
                updated_state.interval,
                updated_state.ease_factor,
                updated_state.repetitions,
                updated_state.next_review,
                updated_state.total_reviews,
                updated_state.correct_streak,
                updated_state.last_review,
                updated_state.card_id,
            ],
        );

        if let Err(e) = result {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Get the most recent review in a deck for undo purposes.
    pub fn get_last_review(&self, deck_id: &str) -> Result<Option<Review>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.card_id, r.quality, r.reviewed_at, r.interval, r.ease_factor, r.repetitions, r.prev_state
             FROM reviews r
             JOIN cards c ON c.id = r.card_id
             WHERE c.deck_id = ?1
             ORDER BY r.reviewed_at DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![deck_id], |row| {
            Ok(Review {
                id: row.get(0)?,
                card_id: row.get(1)?,
                quality: row.get(2)?,
                reviewed_at: row.get(3)?,
                interval: row.get(4)?,
                ease_factor: row.get(5)?,
                repetitions: row.get(6)?,
                prev_state: row.get(7)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// Undo the last review in a deck: restore previous card state and delete the review.
    /// Returns None if there is no review to undo or if the previous state is missing.
    pub fn undo_last_review(&self, deck_id: &str) -> Result<Option<(Card, CardState)>> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;

        let review = match self.get_last_review(deck_id) {
            Ok(Some(r)) => r,
            Ok(None) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Ok(None);
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        };

        // Restore previous state from saved JSON — only then delete the review
        let prev_state: CardState = match review.prev_state {
            Some(ref json) => match serde_json::from_str(json) {
                Ok(s) => s,
                Err(_) => {
                    let _ = self.conn.execute_batch("ROLLBACK");
                    return Ok(None);
                }
            },
            None => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Ok(None);
            }
        };

        if let Err(e) = self.conn.execute(
            "UPDATE card_state SET interval = ?1, ease_factor = ?2, repetitions = ?3, next_review = ?4, total_reviews = ?5, correct_streak = ?6, last_review = ?7 WHERE card_id = ?8",
            params![
                prev_state.interval,
                prev_state.ease_factor,
                prev_state.repetitions,
                prev_state.next_review,
                prev_state.total_reviews,
                prev_state.correct_streak,
                prev_state.last_review,
                prev_state.card_id,
            ],
        ) {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        // Delete the review only after successful state restore
        if let Err(e) = self.conn.execute("DELETE FROM reviews WHERE id = ?1", params![review.id]) {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        self.conn.execute_batch("COMMIT")?;

        // Return card + restored state
        match (self.get_card(&review.card_id)?, self.get_card_state(&review.card_id)?) {
            (Some(card), Some(state)) => Ok(Some((card, state))),
            _ => Ok(None),
        }
    }

    // ── Study Queries ──────────────────────────────────

    /// Returns cards that are due for review (next_review <= today) across one or more decks.
    pub fn get_due_cards(&self, deck_ids: &[String], limit: u32) -> Result<Vec<(Card, CardState)>> {
        let today = Local::now().format("%Y-%m-%d").to_string();

        if deck_ids.is_empty() {
            return Ok(vec![]);
        }

        let n = deck_ids.len();
        let placeholders: Vec<String> = (1..=n).map(|i| format!("?{}", i)).collect();
        let today_param = format!("?{}", n + 1);
        let limit_param = format!("?{}", n + 2);

        let sql = format!(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at,
                    cs.interval, cs.ease_factor, cs.repetitions, cs.next_review,
                    cs.total_reviews, cs.correct_streak, cs.last_review,
                    GROUP_CONCAT(t.name)
             FROM cards c
             JOIN card_state cs ON cs.card_id = c.id
             LEFT JOIN card_tags ct ON ct.card_id = c.id
             LEFT JOIN tags t ON t.id = ct.tag_id
             WHERE c.deck_id IN ({}) AND cs.next_review <= {}
             GROUP BY c.id
             ORDER BY cs.next_review ASC
             LIMIT {}",
            placeholders.join(", "),
            today_param,
            limit_param,
        );

        let mut stmt = self.conn.prepare(&sql)?;

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
        for id in deck_ids {
            param_values.push(Box::new(id.clone()));
        }
        param_values.push(Box::new(today));
        param_values.push(Box::new(limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

        let results = stmt
            .query_map(params_ref.as_slice(), |row| {
                let tags_str: Option<String> = row.get(16)?;
                let tags = tags_str
                    .map(|s| s.split(',').map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                let card = Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    card_type: row.get(2)?,
                    content: row.get(3)?,
                    reasoning: row.get(4)?,
                    front: row.get(5)?,
                    back: row.get(6)?,
                    tags,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                };
                let state = CardState {
                    card_id: row.get(0)?,
                    interval: row.get(9)?,
                    ease_factor: row.get(10)?,
                    repetitions: row.get(11)?,
                    next_review: row.get(12)?,
                    total_reviews: row.get(13)?,
                    correct_streak: row.get(14)?,
                    last_review: row.get(15)?,
                };
                Ok((card, state))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    /// Returns cards that are due for review matching specific tags.
    pub fn get_due_cards_by_tags(&self, tags: &[String], limit: u32) -> Result<Vec<(Card, CardState)>> {
        let today = Local::now().format("%Y-%m-%d").to_string();

        if tags.is_empty() {
            return Ok(vec![]);
        }

        let n = tags.len();
        let placeholders: Vec<String> = (1..=n).map(|i| format!("?{}", i)).collect();
        let today_param = format!("?{}", n + 1);
        let limit_param = format!("?{}", n + 2);

        let sql = format!(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at,
                    cs.interval, cs.ease_factor, cs.repetitions, cs.next_review,
                    cs.total_reviews, cs.correct_streak, cs.last_review,
                    GROUP_CONCAT(t.name)
             FROM cards c
             JOIN card_state cs ON cs.card_id = c.id
             JOIN card_tags ct ON ct.card_id = c.id
             JOIN tags t ON t.id = ct.tag_id
             WHERE t.name IN ({}) AND cs.next_review <= {}
             GROUP BY c.id
             ORDER BY cs.next_review ASC
             LIMIT {}",
            placeholders.join(", "),
            today_param,
            limit_param,
        );

        let mut stmt = self.conn.prepare(&sql)?;

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
        for tag in tags {
            param_values.push(Box::new(tag.clone()));
        }
        param_values.push(Box::new(today));
        param_values.push(Box::new(limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

        let results = stmt
            .query_map(params_ref.as_slice(), |row| {
                let tags_str: Option<String> = row.get(16)?;
                let fetched_tags = tags_str
                    .map(|s| s.split(',').map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                let card = Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    card_type: row.get(2)?,
                    content: row.get(3)?,
                    reasoning: row.get(4)?,
                    front: row.get(5)?,
                    back: row.get(6)?,
                    tags: fetched_tags,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                };
                let state = CardState {
                    card_id: card.id.clone(),
                    interval: row.get(9)?,
                    ease_factor: row.get(10)?,
                    repetitions: row.get(11)?,
                    next_review: row.get(12)?,
                    total_reviews: row.get(13)?,
                    correct_streak: row.get(14)?,
                    last_review: row.get(15)?,
                };
                Ok((card, state))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Count due cards for a deck
    pub fn count_due_cards(&self, deck_id: &str) -> Result<u32> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cards c
             JOIN card_state cs ON cs.card_id = c.id
             WHERE c.deck_id = ?1 AND cs.next_review <= ?2",
            params![deck_id, today],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    // ── Settings ───────────────────────────────────────

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
        match rows.next() {
            Some(v) => Ok(Some(v?)),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Count total cards in a deck
    pub fn count_total_cards(&self, deck_id: &str) -> Result<u32> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cards WHERE deck_id = ?1",
            params![deck_id],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    // ── Stats ─────────────────────────────────────────

    /// Compute UTC timestamp bounds for a local calendar day.
    /// Returns (lower_utc, upper_utc) for use in `reviewed_at >= ? AND reviewed_at < ?`.
    fn day_utc_bounds(local_date: NaiveDate) -> (String, String) {
        let midnight = local_date.and_hms_opt(0, 0, 0).unwrap();
        let next_midnight = (local_date + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap();
        let lower = Local.from_local_datetime(&midnight)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let upper = Local.from_local_datetime(&next_midnight)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        (
            lower.format("%Y-%m-%dT%H:%M:%S").to_string(),
            upper.format("%Y-%m-%dT%H:%M:%S").to_string(),
        )
    }

    pub fn get_deck_stats(&self, deck_id: &str, today_start: &str) -> Result<DeckStats> {
        let agg_result = self.conn.query_row(
            "SELECT
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN cs.total_reviews = 0 THEN 1 ELSE 0 END), 0) as new_cards,
                COALESCE(SUM(CASE WHEN cs.total_reviews > 0 AND cs.interval <= 1 THEN 1 ELSE 0 END), 0) as learning,
                COALESCE(SUM(CASE WHEN cs.total_reviews > 0 AND cs.interval > 1 AND cs.correct_streak < 3 THEN 1 ELSE 0 END), 0) as reviewing,
                COALESCE(SUM(CASE WHEN cs.total_reviews > 0 AND cs.interval > 1 AND cs.correct_streak >= 3 THEN 1 ELSE 0 END), 0) as mastered,
                COALESCE(SUM(CASE WHEN cs.next_review <= ?2 THEN 1 ELSE 0 END), 0) as due_cards,
                COALESCE(AVG(cs.ease_factor), 0.0) as avg_ef,
                COALESCE(AVG(cs.interval), 0.0) as avg_interval,
                COALESCE(SUM(cs.total_reviews), 0) as total_reviews_sum
             FROM cards c
             JOIN card_state cs ON c.id = cs.card_id
             WHERE c.deck_id = ?1",
            params![deck_id, today_start],
            |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, i64>(1)? as u32,
                    row.get::<_, i64>(2)? as u32,
                    row.get::<_, i64>(3)? as u32,
                    row.get::<_, i64>(4)? as u32,
                    row.get::<_, i64>(5)? as u32,
                    row.get::<_, f64>(6)?,
                    row.get::<_, f64>(7)?,
                    row.get::<_, i64>(8)? as u32,
                ))
            },
        );

        let (total, new_cards, learning, reviewing, mastered, due_cards, avg_ef, avg_interval, total_reviews_sum) = agg_result?;

        let today_dt = NaiveDate::parse_from_str(today_start, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().date_naive());
        let (utc_lower, utc_upper) = Self::day_utc_bounds(today_dt);

        let reviews_today: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM reviews r
             JOIN cards c ON r.card_id = c.id
             WHERE c.deck_id = ?1 AND r.reviewed_at >= ?2 AND r.reviewed_at < ?3",
            params![deck_id, utc_lower, utc_upper],
            |row| row.get(0),
        )?;

        Ok(DeckStats {
            total_cards: total,
            due_cards,
            new_cards,
            learning_cards: learning,
            reviewing_cards: reviewing,
            mastered_cards: mastered,
            avg_ease_factor: avg_ef,
            avg_interval,
            total_reviews_sum,
            reviews_today: reviews_today as u32,
        })
    }

    // ── Search ─────────────────────────────────────────

    pub fn search_cards(&self, query: &str) -> Result<Vec<SearchResult>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.card_type, c.content, c.reasoning, c.front, c.back, c.created_at, c.updated_at,
                    d.name as deck_name,
                    GROUP_CONCAT(t.name)
             FROM cards c
             JOIN decks d ON d.id = c.deck_id
             LEFT JOIN card_tags ct ON ct.card_id = c.id
             LEFT JOIN tags t ON t.id = ct.tag_id
             WHERE c.front LIKE ?1 OR c.back LIKE ?1 OR c.reasoning LIKE ?1
             GROUP BY c.id
             ORDER BY c.updated_at DESC
             LIMIT 50",
        )?;
        let results = stmt
            .query_map(params![pattern], |row| {
                let tags_str: Option<String> = row.get(10)?;
                let tags = tags_str
                    .map(|s| s.split(',').map(|t| t.to_string()).collect())
                    .unwrap_or_default();

                Ok(SearchResult {
                    card: Card {
                        id: row.get(0)?,
                        deck_id: row.get(1)?,
                        card_type: row.get(2)?,
                        content: row.get(3)?,
                        reasoning: row.get(4)?,
                        front: row.get(5)?,
                        back: row.get(6)?,
                        tags,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    },
                    deck_name: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(results)
    }

    // ── Dashboard ─────────────────────────────────────

    pub fn get_dashboard_stats(&self, today: &str) -> Result<DashboardStats> {
        let agg = self.conn.query_row(
            "SELECT
                COALESCE(COUNT(*), 0),
                COALESCE(SUM(CASE WHEN cs.next_review <= ?1 THEN 1 ELSE 0 END), 0),
                COALESCE(AVG(cs.ease_factor), 0.0)
             FROM cards c
             JOIN card_state cs ON c.id = cs.card_id",
            params![today],
            |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, i64>(1)? as u32,
                    row.get::<_, f64>(2)?,
                ))
            },
        )?;

        let (total_cards, due_cards, avg_ease_factor) = agg;

        let today_dt = NaiveDate::parse_from_str(today, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().date_naive());
        let (utc_lower, utc_upper) = Self::day_utc_bounds(today_dt);

        let reviews_today: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM reviews WHERE reviewed_at >= ?1 AND reviewed_at < ?2",
            params![utc_lower, utc_upper],
            |row| row.get(0),
        )?;

        // Compute streak: consecutive days with at least one review going back from today.
        // Each local day is checked via UTC bounds so DST transitions are handled correctly.
        let mut streak = 0u32;
        let mut streak_stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM reviews WHERE reviewed_at >= ?1 AND reviewed_at < ?2",
        )?;

        for i in 0i64..366 {
            let check_date = today_dt - Duration::days(i);
            let (check_lower, check_upper) = Self::day_utc_bounds(check_date);
            let has_review: bool = streak_stmt
                .query_row(params![check_lower, check_upper], |row| row.get::<_, i64>(0))
                .map(|c| c > 0)
                .unwrap_or(false);
            if has_review {
                streak += 1;
            } else if i == 0 {
                // Today may not have any reviews yet, keep checking
                continue;
            } else {
                break;
            }
        }

        Ok(DashboardStats {
            total_cards,
            due_cards,
            reviews_today: reviews_today as u32,
            avg_ease_factor,
            streak_days: streak,
        })
    }
    // ── Exams ──────────────────────────────────────────

    pub fn create_exam(&self, name: &str, exam_type: &str, exam_date: &str, deck_ids: Vec<String>) -> Result<crate::db::models::Exam> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO exams (id, name, exam_type, exam_date, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, exam_type, exam_date, now],
        )?;

        for deck_id in &deck_ids {
            self.conn.execute(
                "INSERT INTO exam_decks (exam_id, deck_id) VALUES (?1, ?2)",
                params![id, deck_id],
            )?;
        }

        Ok(crate::db::models::Exam {
            id,
            name: name.to_string(),
            exam_type: exam_type.to_string(),
            exam_date: exam_date.to_string(),
            created_at: now,
            deck_ids,
        })
    }

    pub fn list_exams(&self) -> Result<Vec<crate::db::models::Exam>> {
        let mut stmt = self.conn.prepare("SELECT id, name, exam_type, exam_date, created_at FROM exams ORDER BY exam_date ASC")?;
        let mut exams = Vec::new();
        
        let exam_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        for row in exam_rows {
            let (id, name, exam_type, exam_date, created_at) = row?;
            let mut deck_stmt = self.conn.prepare("SELECT deck_id FROM exam_decks WHERE exam_id = ?1")?;
            let deck_ids: Result<Vec<String>> = deck_stmt.query_map(params![id], |r| r.get(0))?.collect();
            
            exams.push(crate::db::models::Exam {
                id,
                name,
                exam_type,
                exam_date,
                created_at,
                deck_ids: deck_ids?,
            });
        }

        Ok(exams)
    }

    pub fn delete_exam(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM exams WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_exam(&self, id: &str, name: &str, exam_type: &str, exam_date: &str, deck_ids: Vec<String>) -> Result<crate::db::models::Exam> {
        self.conn.execute(
            "UPDATE exams SET name = ?1, exam_type = ?2, exam_date = ?3 WHERE id = ?4",
            params![name, exam_type, exam_date, id],
        )?;

        self.conn.execute("DELETE FROM exam_decks WHERE exam_id = ?1", params![id])?;

        for deck_id in &deck_ids {
            self.conn.execute(
                "INSERT INTO exam_decks (exam_id, deck_id) VALUES (?1, ?2)",
                params![id, deck_id],
            )?;
        }

        let created_at: String = self.conn.query_row(
            "SELECT created_at FROM exams WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ).unwrap_or_default();

        Ok(crate::db::models::Exam {
            id: id.to_string(),
            name: name.to_string(),
            exam_type: exam_type.to_string(),
            exam_date: exam_date.to_string(),
            created_at,
            deck_ids,
        })
    }

    pub fn get_exam_stats(&self, id: &str) -> Result<crate::db::models::ExamStats> {
        // Find the exam date
        let exam_date_str: String = self.conn.query_row(
            "SELECT exam_date FROM exams WHERE id = ?1",
            params![id],
            |row| row.get(0)
        )?;

        let today = Local::now().naive_local().date();
        let target_date = NaiveDate::parse_from_str(&exam_date_str, "%Y-%m-%d")
            .unwrap_or(today);
        
        let days_left = (target_date - today).num_days() as i32;

        // Find the cards linked to this exam's decks
        // A card is mastered if interval > 21 days
        let total_cards: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM cards c
             JOIN exam_decks ed ON c.deck_id = ed.deck_id
             WHERE ed.exam_id = ?1",
            params![id],
            |row| row.get(0)
        )?;

        let mastered_cards: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM cards c
             JOIN exam_decks ed ON c.deck_id = ed.deck_id
             JOIN card_state cs ON c.id = cs.card_id
             WHERE ed.exam_id = ?1 AND cs.interval >= 21",
            params![id],
            |row| row.get(0)
        )?;

        let cards_left = total_cards.saturating_sub(mastered_cards);
        
        let effective_days = if days_left > 0 { days_left as u32 } else { 1 };
        let cards_per_day = (cards_left as f32 / effective_days as f32).ceil() as u32;

        Ok(crate::db::models::ExamStats {
            total_cards,
            mastered_cards,
            cards_left,
            days_left,
            cards_per_day,
        })
    }

    // ── Test Engine Repository Methods ─────────────────────────

    pub fn create_exam_template(
        &self,
        name: &str,
        deck_ids: Vec<String>,
        tags: Vec<String>,
        allowed_types: Vec<String>,
        question_count: u32,
        time_limit_minutes: u32,
        pass_percentage: f64,
        seed: Option<u64>,
    ) -> Result<crate::db::models::ExamTemplate> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let deck_ids_json = serde_json::to_string(&deck_ids).unwrap_or_else(|_| "[]".to_string());
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let allowed_types_json = serde_json::to_string(&allowed_types).unwrap_or_else(|_| "[]".to_string());

        self.conn.execute(
            "INSERT INTO exam_templates (id, name, deck_ids_json, tags_json, allowed_types_json, question_count, time_limit_minutes, pass_percentage, seed, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                name,
                deck_ids_json,
                tags_json,
                allowed_types_json,
                question_count,
                time_limit_minutes,
                pass_percentage,
                seed,
                now
            ],
        )?;

        Ok(crate::db::models::ExamTemplate {
            id,
            name: name.to_string(),
            deck_ids,
            tags,
            allowed_card_types: allowed_types,
            question_count,
            time_limit_minutes,
            pass_percentage,
            seed,
            created_at: now,
        })
    }

    pub fn list_exam_templates(&self) -> Result<Vec<crate::db::models::ExamTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, deck_ids_json, tags_json, allowed_types_json, question_count, time_limit_minutes, pass_percentage, seed, created_at
             FROM exam_templates ORDER BY created_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let deck_ids_json: String = row.get(2)?;
            let tags_json: String = row.get(3)?;
            let allowed_types_json: String = row.get(4)?;
            Ok(crate::db::models::ExamTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                deck_ids: serde_json::from_str(&deck_ids_json).unwrap_or_default(),
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                allowed_card_types: serde_json::from_str(&allowed_types_json).unwrap_or_default(),
                question_count: row.get(5)?,
                time_limit_minutes: row.get(6)?,
                pass_percentage: row.get(7)?,
                seed: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn get_exam_template(&self, id: &str) -> Result<Option<crate::db::models::ExamTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, deck_ids_json, tags_json, allowed_types_json, question_count, time_limit_minutes, pass_percentage, seed, created_at
             FROM exam_templates WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let deck_ids_json: String = row.get(2)?;
            let tags_json: String = row.get(3)?;
            let allowed_types_json: String = row.get(4)?;
            Ok(Some(crate::db::models::ExamTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                deck_ids: serde_json::from_str(&deck_ids_json).unwrap_or_default(),
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                allowed_card_types: serde_json::from_str(&allowed_types_json).unwrap_or_default(),
                question_count: row.get(5)?,
                time_limit_minutes: row.get(6)?,
                pass_percentage: row.get(7)?,
                seed: row.get(8)?,
                created_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_exam_template(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM exam_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn start_exam_session(
        &self,
        template_id: Option<String>,
        name: &str,
        deck_ids: Vec<String>,
        tags: Vec<String>,
        allowed_types: Vec<String>,
        question_count: u32,
        custom_seed: Option<u64>,
    ) -> Result<(crate::db::models::ExamSession, Vec<crate::db::models::ExamQuestion>)> {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;

        // If template_id provided and no custom seed, check template for seed
        let mut resolved_seed = custom_seed;
        if resolved_seed.is_none() {
            if let Some(ref t_id) = template_id {
                if let Ok(Some(tmpl)) = self.get_exam_template(t_id) {
                    resolved_seed = tmpl.seed;
                }
            }
        }

        let seed = resolved_seed.unwrap_or_else(|| rand::random::<u64>());
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        // 1. Query matching cards
        let all_cards = self.list_all_cards()?;
        let mut eligible_cards: Vec<Card> = all_cards
            .into_iter()
            .filter(|c| {
                if !deck_ids.is_empty() && !deck_ids.contains(&c.deck_id) {
                    return false;
                }
                if !allowed_types.is_empty() && !allowed_types.contains(&c.card_type) {
                    return false;
                }
                if !tags.is_empty() && !tags.iter().any(|t| c.tags.contains(t)) {
                    return false;
                }
                true
            })
            .collect();

        // 2. Deterministic shuffle using seed
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        eligible_cards.shuffle(&mut rng);

        if eligible_cards.len() > question_count as usize {
            eligible_cards.truncate(question_count as usize);
        }

        // Transaction protection
        self.conn.execute_batch("BEGIN TRANSACTION;")?;

        // 3. Insert ExamSession
        if let Err(e) = self.conn.execute(
            "INSERT INTO exam_sessions (id, template_id, name, status, started_at, seed, current_index, created_at)
             VALUES (?1, ?2, ?3, 'in_progress', ?4, ?5, 0, ?4)",
            params![session_id, template_id, name, now, seed],
        ) {
            let _ = self.conn.execute_batch("ROLLBACK;");
            return Err(e);
        }

        // 4. Create immutable snapshots in exam_questions
        let mut questions = Vec::new();
        for (idx, card) in eligible_cards.iter().enumerate() {
            let q_id = Uuid::new_v4().to_string();
            let prompt = card.front.clone();
            let expected_answer = card.back.clone();

            if let Err(e) = self.conn.execute(
                "INSERT INTO exam_questions (id, session_id, card_id, question_index, card_type, prompt, options_json, expected_answer, points_earned, max_points)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0.0, 1.0)",
                params![q_id, session_id, card.id, idx as u32, card.card_type, prompt, card.content, expected_answer],
            ) {
                let _ = self.conn.execute_batch("ROLLBACK;");
                return Err(e);
            }

            questions.push(crate::db::models::ExamQuestion {
                id: q_id,
                session_id: session_id.clone(),
                card_id: card.id.clone(),
                question_index: idx as u32,
                card_type: card.card_type.clone(),
                prompt,
                options_json: card.content.clone(),
                expected_answer,
                user_answer: None,
                is_correct: None,
                points_earned: 0.0,
                max_points: 1.0,
            });
        }

        self.conn.execute_batch("COMMIT;")?;

        let session = crate::db::models::ExamSession {
            id: session_id,
            template_id,
            name: name.to_string(),
            status: "in_progress".to_string(),
            started_at: now.clone(),
            finished_at: None,
            seed,
            current_index: 0,
            created_at: now,
        };

        Ok((session, questions))
    }

    pub fn get_exam_session_with_questions(&self, session_id: &str) -> Result<Option<(crate::db::models::ExamSession, Vec<crate::db::models::ExamQuestion>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template_id, name, status, started_at, finished_at, seed, current_index, created_at
             FROM exam_sessions WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let session = crate::db::models::ExamSession {
                id: row.get(0)?,
                template_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
                started_at: row.get(4)?,
                finished_at: row.get(5)?,
                seed: row.get(6)?,
                current_index: row.get(7)?,
                created_at: row.get(8)?,
            };

            let mut q_stmt = self.conn.prepare(
                "SELECT id, session_id, card_id, question_index, card_type, prompt, options_json, expected_answer, user_answer, is_correct, points_earned, max_points
                 FROM exam_questions WHERE session_id = ?1 ORDER BY question_index ASC"
            )?;

            let questions: Vec<crate::db::models::ExamQuestion> = q_stmt
                .query_map(params![session_id], |qrow| {
                    let is_corr_int: Option<i32> = qrow.get(9)?;
                    Ok(crate::db::models::ExamQuestion {
                        id: qrow.get(0)?,
                        session_id: qrow.get(1)?,
                        card_id: qrow.get(2)?,
                        question_index: qrow.get(3)?,
                        card_type: qrow.get(4)?,
                        prompt: qrow.get(5)?,
                        options_json: qrow.get(6)?,
                        expected_answer: qrow.get(7)?,
                        user_answer: qrow.get(8)?,
                        is_correct: is_corr_int.map(|v| v == 1),
                        points_earned: qrow.get(10)?,
                        max_points: qrow.get(11)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some((session, questions)))
        } else {
            Ok(None)
        }
    }

    pub fn submit_exam_question_answer(
        &self,
        question_id: &str,
        user_answer: &str,
    ) -> Result<crate::db::models::ExamQuestion> {
        let (card_type, expected_back, options_json): (String, String, Option<String>) = self.conn.query_row(
            "SELECT card_type, expected_answer, options_json FROM exam_questions WHERE id = ?1",
            params![question_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        let (is_correct, points_earned) = crate::test_engine::evaluate_answer(
            &card_type,
            &expected_back,
            options_json.as_deref(),
            user_answer,
        );

        self.conn.execute(
            "UPDATE exam_questions SET user_answer = ?1, is_correct = ?2, points_earned = ?3 WHERE id = ?4",
            params![user_answer, if is_correct { 1 } else { 0 }, points_earned, question_id],
        )?;

        let q = self.conn.query_row(
            "SELECT id, session_id, card_id, question_index, card_type, prompt, options_json, expected_answer, user_answer, is_correct, points_earned, max_points
             FROM exam_questions WHERE id = ?1",
            params![question_id],
            |row| {
                let is_corr_int: Option<i32> = row.get(9)?;
                Ok(crate::db::models::ExamQuestion {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    card_id: row.get(2)?,
                    question_index: row.get(3)?,
                    card_type: row.get(4)?,
                    prompt: row.get(5)?,
                    options_json: row.get(6)?,
                    expected_answer: row.get(7)?,
                    user_answer: row.get(8)?,
                    is_correct: is_corr_int.map(|v| v == 1),
                    points_earned: row.get(10)?,
                    max_points: row.get(11)?,
                })
            },
        )?;

        Ok(q)
    }

    pub fn finish_exam_session(
        &self,
        session_id: &str,
        fallback_pass_percentage: f64,
    ) -> Result<crate::db::models::ExamResult> {
        let (started_at_str, template_id): (String, Option<String>) = self.conn.query_row(
            "SELECT started_at, template_id FROM exam_sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let pass_percentage = if let Some(t_id) = template_id {
            if let Ok(Some(tmpl)) = self.get_exam_template(&t_id) {
                tmpl.pass_percentage
            } else {
                fallback_pass_percentage
            }
        } else {
            fallback_pass_percentage
        };

        let now_dt = Utc::now();
        let now = now_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        let started_dt = NaiveDateTime::parse_from_str(&started_at_str, "%Y-%m-%dT%H:%M:%S")
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            .unwrap_or(now_dt);

        let duration_seconds = (now_dt - started_dt).num_seconds().max(0) as u64;

        self.conn.execute(
            "UPDATE exam_sessions SET status = 'completed', finished_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;

        // Fetch questions for analysis
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, card_id, question_index, card_type, prompt, options_json, expected_answer, user_answer, is_correct, points_earned, max_points
             FROM exam_questions WHERE session_id = ?1 ORDER BY question_index ASC"
        )?;

        let questions: Vec<crate::db::models::ExamQuestion> = stmt
            .query_map(params![session_id], |row| {
                let is_corr_int: Option<i32> = row.get(9)?;
                Ok(crate::db::models::ExamQuestion {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    card_id: row.get(2)?,
                    question_index: row.get(3)?,
                    card_type: row.get(4)?,
                    prompt: row.get(5)?,
                    options_json: row.get(6)?,
                    expected_answer: row.get(7)?,
                    user_answer: row.get(8)?,
                    is_correct: is_corr_int.map(|v| v == 1),
                    points_earned: row.get(10)?,
                    max_points: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let total_questions = questions.len() as u32;
        let mut correct_count = 0;
        let mut incorrect_count = 0;
        let mut skipped_count = 0;
        let mut total_points = 0.0;

        use std::collections::HashMap;
        let mut card_type_stats: HashMap<String, (u32, u32)> = HashMap::new();

        for q in &questions {
            let entry = card_type_stats.entry(q.card_type.clone()).or_insert((0, 0));
            entry.0 += 1;

            match q.is_correct {
                Some(true) => {
                    correct_count += 1;
                    total_points += q.points_earned;
                    entry.1 += 1;
                }
                Some(false) => {
                    incorrect_count += 1;
                    total_points += q.points_earned;
                }
                None => skipped_count += 1,
            }
        }

        let score_percentage = if total_questions > 0 {
            (total_points / total_questions as f64) * 100.0
        } else {
            0.0
        };

        let passed = score_percentage >= pass_percentage;

        let by_card_type: Vec<crate::db::models::CategoryScore> = card_type_stats
            .into_iter()
            .map(|(key, (total, correct))| crate::db::models::CategoryScore {
                key,
                total,
                correct,
                percentage: if total > 0 { (correct as f64 / total as f64) * 100.0 } else { 0.0 },
            })
            .collect();

        let breakdown = crate::db::models::ExamResultBreakdown {
            by_deck: vec![],
            by_tag: vec![],
            by_card_type,
        };
        let breakdown_json = serde_json::to_string(&breakdown).unwrap_or_else(|_| "{}".to_string());

        self.conn.execute(
            "INSERT INTO exam_results (session_id, score_percentage, passed, total_questions, correct_count, incorrect_count, skipped_count, duration_seconds, breakdown_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(session_id) DO UPDATE SET
               score_percentage = excluded.score_percentage,
               passed = excluded.passed,
               correct_count = excluded.correct_count,
               duration_seconds = excluded.duration_seconds,
               breakdown_json = excluded.breakdown_json",
            params![
                session_id,
                score_percentage,
                if passed { 1 } else { 0 },
                total_questions,
                correct_count,
                incorrect_count,
                skipped_count,
                duration_seconds,
                breakdown_json
            ],
        )?;

        Ok(crate::db::models::ExamResult {
            session_id: session_id.to_string(),
            score_percentage,
            passed,
            total_questions,
            correct_count,
            incorrect_count,
            skipped_count,
            duration_seconds,
            breakdown,
        })
    }

    pub fn get_exam_result(&self, session_id: &str) -> Result<Option<crate::db::models::ExamResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, score_percentage, passed, total_questions, correct_count, incorrect_count, skipped_count, duration_seconds, breakdown_json
             FROM exam_results WHERE session_id = ?1"
        )?;

        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let passed_int: i32 = row.get(2)?;
            let breakdown_json: String = row.get(8)?;
            Ok(Some(crate::db::models::ExamResult {
                session_id: row.get(0)?,
                score_percentage: row.get(1)?,
                passed: passed_int == 1,
                total_questions: row.get(3)?,
                correct_count: row.get(4)?,
                incorrect_count: row.get(5)?,
                skipped_count: row.get(6)?,
                duration_seconds: row.get(7)?,
                breakdown: serde_json::from_str(&breakdown_json).unwrap_or(crate::db::models::ExamResultBreakdown {
                    by_deck: vec![],
                    by_tag: vec![],
                    by_card_type: vec![],
                }),
            }))
        } else {
            Ok(None)
        }
    }
}

