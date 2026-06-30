use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use chrono::Utc;

use super::models::{Card, CardState, Deck, Review};

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
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                review.id,
                review.card_id,
                review.quality,
                review.reviewed_at,
                review.interval,
                review.ease_factor,
                review.repetitions,
            ],
        )?;
        Ok(())
    }

    // ── Study Queries ──────────────────────────────────

    /// Returns cards that are due for review (next_review <= today)
    pub fn get_due_cards(&self, deck_id: &str, limit: u32) -> Result<Vec<(Card, CardState)>> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

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
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cards c
             JOIN card_state cs ON cs.card_id = c.id
             WHERE c.deck_id = ?1 AND cs.next_review <= ?2",
            params![deck_id, today],
            |row| row.get(0),
        )?;
        Ok(count as u32)
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
}
