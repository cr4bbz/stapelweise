use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use chrono::{Utc, Local, TimeZone, NaiveDate, Duration};

use super::models::{Card, CardState, DashboardStats, Deck, DeckStats, Review};

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

    // ── Cards ──────────────────────────────────────────

    pub fn create_card(&self, deck_id: &str, front: &str, back: &str) -> Result<Card> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO cards (id, deck_id, front, back, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, deck_id, front, back, now, now],
        )?;

        // Create initial card_state for this card
        self.conn.execute(
            "INSERT INTO card_state (card_id) VALUES (?1)",
            params![id],
        )?;

        Ok(Card {
            id,
            deck_id: deck_id.to_string(),
            front: front.to_string(),
            back: back.to_string(),
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
        state: &CardState,
    ) -> Result<Card> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        self.conn.execute(
            "INSERT INTO cards (id, deck_id, front, back, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, deck_id, front, back, now, now],
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

        Ok(Card {
            id,
            deck_id: deck_id.to_string(),
            front: front.to_string(),
            back: back.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_cards(&self, deck_id: &str) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, deck_id, front, back, created_at, updated_at FROM cards WHERE deck_id = ?1 ORDER BY created_at DESC",
        )?;

        let cards = stmt
            .query_map(params![deck_id], |row| {
                Ok(Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    front: row.get(2)?,
                    back: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(cards)
    }

    pub fn get_card(&self, card_id: &str) -> Result<Option<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, deck_id, front, back, created_at, updated_at FROM cards WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![card_id], |row| {
            Ok(Card {
                id: row.get(0)?,
                deck_id: row.get(1)?,
                front: row.get(2)?,
                back: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        match rows.next() {
            Some(card) => Ok(Some(card?)),
            None => Ok(None),
        }
    }

    pub fn update_card(&self, card_id: &str, front: &str, back: &str) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.conn.execute(
            "UPDATE cards SET front = ?1, back = ?2, updated_at = ?3 WHERE id = ?4",
            params![front, back, now, card_id],
        )?;
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
        let review = match self.get_last_review(deck_id)? {
            Some(r) => r,
            None => return Ok(None),
        };

        // Restore previous state from saved JSON — only then delete the review
        let prev_state: CardState = match review.prev_state {
            Some(ref json) => match serde_json::from_str(json) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            },
            None => return Ok(None),
        };

        self.conn.execute(
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
        )?;

        // Delete the review only after successful state restore
        self.conn
            .execute("DELETE FROM reviews WHERE id = ?1", params![review.id])?;

        // Return card + restored state
        match (self.get_card(&review.card_id)?, self.get_card_state(&review.card_id)?) {
            (Some(card), Some(state)) => Ok(Some((card, state))),
            _ => Ok(None),
        }
    }

    // ── Study Queries ──────────────────────────────────

    /// Returns cards that are due for review (next_review <= today)
    pub fn get_due_cards(&self, deck_id: &str, limit: u32) -> Result<Vec<(Card, CardState)>> {
        let today = Local::now().format("%Y-%m-%d").to_string();

        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.deck_id, c.front, c.back, c.created_at, c.updated_at,
                    cs.interval, cs.ease_factor, cs.repetitions, cs.next_review,
                    cs.total_reviews, cs.correct_streak, cs.last_review
             FROM cards c
             JOIN card_state cs ON cs.card_id = c.id
             WHERE c.deck_id = ?1 AND cs.next_review <= ?2
             ORDER BY cs.next_review ASC
             LIMIT ?3",
        )?;

        let results = stmt
            .query_map(params![deck_id, today, limit], |row| {
                let card = Card {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    front: row.get(2)?,
                    back: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                };
                let state = CardState {
                    card_id: row.get(0)?,
                    interval: row.get(6)?,
                    ease_factor: row.get(7)?,
                    repetitions: row.get(8)?,
                    next_review: row.get(9)?,
                    total_reviews: row.get(10)?,
                    correct_streak: row.get(11)?,
                    last_review: row.get(12)?,
                };
                Ok((card, state))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    /// Count due cards in a deck
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
}
