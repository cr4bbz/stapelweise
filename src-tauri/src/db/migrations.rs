use rusqlite::Connection;

const LATEST_VERSION: i32 = 3;

/// Run all pending migrations to bring the database up to `LATEST_VERSION`.
///
/// Uses `PRAGMA user_version` to track the applied schema version.
/// For existing databases that predate version tracking (user_version = 0),
/// the current schema is detected via introspection so that only missing
/// migrations are applied.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    let version = if current_version == 0 {
        detect_existing_schema_version(conn)?
    } else {
        current_version
    };

    if version < 1 {
        apply_v1(conn)?;
    }
    if version < 2 {
        apply_v2(conn)?;
    }
    if version < 3 {
        apply_v3(conn)?;
    }

    Ok(())
}

/// Detect the schema version of an existing database that has no `user_version` set.
///
/// Checks for the presence of tables and columns from later migrations to determine
/// how far the old ad-hoc migration runner already got.
fn detect_existing_schema_version(conn: &Connection) -> Result<i32, rusqlite::Error> {
    let tables_exist: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('decks', 'cards', 'reviews', 'card_state')",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|c| c >= 4)?;

    if !tables_exist {
        // Fresh database — let the migration runner create everything
        return Ok(0);
    }

    // V1 tables exist. Check for V2 (settings table).
    let settings_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'settings'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|c| c > 0)?;

    if !settings_exists {
        return Ok(1);
    }

    // V2 exists. Check for V3 (reviews.prev_state column).
    let has_prev_state: bool = conn
        .prepare("SELECT prev_state FROM reviews LIMIT 0")
        .is_ok();

    if !has_prev_state {
        return Ok(2);
    }

    // All migrations already applied — set user_version to latest
    conn.pragma_update(None, "user_version", LATEST_VERSION)?;
    Ok(LATEST_VERSION)
}

fn apply_v1(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS decks (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cards (
            id          TEXT PRIMARY KEY,
            deck_id     TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
            front       TEXT NOT NULL,
            back        TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS reviews (
            id           TEXT PRIMARY KEY,
            card_id      TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
            quality      INTEGER NOT NULL,
            reviewed_at  TEXT NOT NULL,
            interval     INTEGER NOT NULL,
            ease_factor  REAL NOT NULL,
            repetitions  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS card_state (
            card_id         TEXT PRIMARY KEY REFERENCES cards(id) ON DELETE CASCADE,
            interval        INTEGER NOT NULL DEFAULT 0,
            ease_factor     REAL NOT NULL DEFAULT 2.5,
            repetitions     INTEGER NOT NULL DEFAULT 0,
            next_review     TEXT NOT NULL DEFAULT '1970-01-01',
            total_reviews   INTEGER NOT NULL DEFAULT 0,
            correct_streak  INTEGER NOT NULL DEFAULT 0,
            last_review     TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_cards_deck ON cards(deck_id);
        CREATE INDEX IF NOT EXISTS idx_reviews_card ON reviews(card_id);
        CREATE INDEX IF NOT EXISTS idx_card_state_next ON card_state(next_review);",
    )?;

    conn.pragma_update(None, "user_version", 1)?;
    Ok(())
}

fn apply_v2(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;

    conn.pragma_update(None, "user_version", 2)?;
    Ok(())
}

fn apply_v3(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Only add the column if it doesn't already exist
    let has_column: bool = conn
        .prepare("SELECT prev_state FROM reviews LIMIT 0")
        .is_ok();

    if !has_column {
        conn.execute("ALTER TABLE reviews ADD COLUMN prev_state TEXT;", [])?;
    }

    conn.pragma_update(None, "user_version", 3)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_fresh_database() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_VERSION);

        // Verify all tables exist
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('decks', 'cards', 'reviews', 'card_state', 'settings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 5);

        // Verify prev_state column exists
        conn.execute("INSERT INTO decks (id, name, created_at, updated_at) VALUES ('d1', 'test', '', '')", [])
            .unwrap();
        conn.execute("INSERT INTO cards (id, deck_id, front, back, created_at, updated_at) VALUES ('c1', 'd1', '', '', '', '')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO reviews (id, card_id, quality, reviewed_at, interval, ease_factor, repetitions, prev_state) VALUES ('r1', 'c1', 3, '', 0, 2.5, 0, NULL)",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_idempotent_rerun() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_VERSION);
    }

    #[test]
    fn test_partial_migration_v1_to_latest() {
        let conn = Connection::open_in_memory().unwrap();

        // Simulate a V1-only database
        apply_v1(&conn).unwrap();
        // Reset user_version to simulate old DB
        conn.pragma_update(None, "user_version", 1).unwrap();

        // Now run the full migrator — should apply V2 and V3
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_VERSION);

        // Settings table should exist
        let settings_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(settings_exists, 1);
    }

    #[test]
    fn test_existing_db_detection_v1() {
        let conn = Connection::open_in_memory().unwrap();

        // Apply only V1 tables manually (no user_version set)
        conn.execute_batch(
            "CREATE TABLE decks (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
             CREATE TABLE cards (id TEXT PRIMARY KEY, deck_id TEXT NOT NULL, front TEXT NOT NULL, back TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
             CREATE TABLE reviews (id TEXT PRIMARY KEY, card_id TEXT NOT NULL, quality INTEGER NOT NULL, reviewed_at TEXT NOT NULL, interval INTEGER NOT NULL, ease_factor REAL NOT NULL, repetitions INTEGER NOT NULL);
             CREATE TABLE card_state (card_id TEXT PRIMARY KEY, interval INTEGER NOT NULL DEFAULT 0, ease_factor REAL NOT NULL DEFAULT 2.5, repetitions INTEGER NOT NULL DEFAULT 0, next_review TEXT NOT NULL DEFAULT '1970-01-01', total_reviews INTEGER NOT NULL DEFAULT 0, correct_streak INTEGER NOT NULL DEFAULT 0, last_review TEXT);",
        ).unwrap();

        // Should detect V1 and apply V2+V3
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_VERSION);
    }
}
