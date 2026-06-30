use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[
    // V1: Initial schema
    "CREATE TABLE IF NOT EXISTS decks (
        id          TEXT PRIMARY KEY,
        name        TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS cards (
        id          TEXT PRIMARY KEY,
        deck_id     TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
        front       TEXT NOT NULL,
        back        TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS reviews (
        id           TEXT PRIMARY KEY,
        card_id      TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
        quality      INTEGER NOT NULL,
        reviewed_at  TEXT NOT NULL,
        interval     INTEGER NOT NULL,
        ease_factor  REAL NOT NULL,
        repetitions  INTEGER NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS card_state (
        card_id         TEXT PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
        interval        INTEGER NOT NULL DEFAULT 0,
        ease_factor     REAL NOT NULL DEFAULT 2.5,
        repetitions     INTEGER NOT NULL DEFAULT 0,
        next_review     TEXT NOT NULL DEFAULT '1970-01-01',
        total_reviews   INTEGER NOT NULL DEFAULT 0,
        correct_streak  INTEGER NOT NULL DEFAULT 0,
        last_review     TEXT
    );",
    "CREATE INDEX IF NOT EXISTS idx_cards_deck ON cards(deck_id);",
    "CREATE INDEX IF NOT EXISTS idx_reviews_card ON reviews(card_id);",
    "CREATE INDEX IF NOT EXISTS idx_card_state_next ON card_state(next_review);",
    // V2: User settings
    "CREATE TABLE IF NOT EXISTS settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );",
];

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    for migration in MIGRATIONS {
        conn.execute(migration, [])?;
    }

    Ok(())
}
