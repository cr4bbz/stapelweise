use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use chrono::{Utc, Local, TimeZone, NaiveDate, Duration};

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

    /// Insert a card with a specific card_state (for seed data / testing).
    /// Unlike create_card, this does NOT use default card_state values.
    pub fn seed_insert_card_with_state(
        &self,
        deck_id: &str,
        front: &str,
        back: &str,
        tags: Vec<String>,
        state: &CardState,
    ) -> Result<Card> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO cards (id, deck_id, card_type, content, reasoning, front, back, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, deck_id, "basic", rusqlite::types::Null, rusqlite::types::Null, front, back, now, now],
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
            card_type: "basic".to_string(),
            content: None,
            reasoning: None,
            front: front.to_string(),
            back: back.to_string(),
            tags: vec![],
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
}
