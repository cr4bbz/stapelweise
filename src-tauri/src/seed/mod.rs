use crate::db::models::{CardState, Deck};
use crate::db::repository::Repository;
use chrono::{NaiveDate, Utc};

/// SM-2 scenario variants for seed data
enum Scenario {
    New,
    DueTodayFirst,
    DueTodayStruggling,
    DueTomorrow,
    DueIn7Days,
    DueIn30Days,
    Overdue,
    Graduated,
}

impl Scenario {
    fn build_state(&self, card_id: &str, today: NaiveDate) -> CardState {
        let fmt = |d: NaiveDate| -> String { d.format("%Y-%m-%d").to_string() };
        let yday = fmt(today - chrono::Duration::days(1));

        match self {
            Scenario::New => CardState {
                card_id: card_id.to_string(),
                interval: 0,
                ease_factor: 2.5,
                repetitions: 0,
                next_review: "1970-01-01".to_string(),
                total_reviews: 0,
                correct_streak: 0,
                last_review: None,
            },
            Scenario::DueTodayFirst => CardState {
                card_id: card_id.to_string(),
                interval: 1,
                ease_factor: 2.5,
                repetitions: 1,
                next_review: fmt(today),
                total_reviews: 1,
                correct_streak: 1,
                last_review: Some(yday),
            },
            Scenario::DueTodayStruggling => CardState {
                card_id: card_id.to_string(),
                interval: 1,
                ease_factor: 1.3,
                repetitions: 3,
                next_review: fmt(today),
                total_reviews: 5,
                correct_streak: 0,
                last_review: Some(yday),
            },
            Scenario::DueTomorrow => CardState {
                card_id: card_id.to_string(),
                interval: 6,
                ease_factor: 2.6,
                repetitions: 2,
                next_review: fmt(today + chrono::Duration::days(1)),
                total_reviews: 2,
                correct_streak: 2,
                last_review: Some(fmt(today - chrono::Duration::days(5))),
            },
            Scenario::DueIn7Days => CardState {
                card_id: card_id.to_string(),
                interval: 7,
                ease_factor: 2.5,
                repetitions: 3,
                next_review: fmt(today + chrono::Duration::days(7)),
                total_reviews: 3,
                correct_streak: 3,
                last_review: Some(fmt(today)),
            },
            Scenario::DueIn30Days => CardState {
                card_id: card_id.to_string(),
                interval: 30,
                ease_factor: 2.7,
                repetitions: 5,
                next_review: fmt(today + chrono::Duration::days(30)),
                total_reviews: 5,
                correct_streak: 5,
                last_review: Some(fmt(today - chrono::Duration::days(30))),
            },
            Scenario::Overdue => CardState {
                card_id: card_id.to_string(),
                interval: 1,
                ease_factor: 1.5,
                repetitions: 5,
                next_review: fmt(today - chrono::Duration::days(3)),
                total_reviews: 6,
                correct_streak: 0,
                last_review: Some(fmt(today - chrono::Duration::days(4))),
            },
            Scenario::Graduated => CardState {
                card_id: card_id.to_string(),
                interval: 30,
                ease_factor: 2.7,
                repetitions: 5,
                next_review: fmt(today + chrono::Duration::days(30)),
                total_reviews: 5,
                correct_streak: 5,
                last_review: Some(fmt(today - chrono::Duration::days(30))),
            },
        }
    }
}

pub struct SeedGenerator;

impl SeedGenerator {
    /// Generate 3 sample decks with cards in varied SM-2 states.
    /// Returns all decks so the frontend can display them immediately.
    fn build_deck(repo: &Repository, name: &str, cards: Vec<(&str, &str, Scenario, Vec<&str>)>, today: NaiveDate, skip: &[String]) -> Result<Option<Deck>, String> {
        if skip.contains(&name.to_string()) {
            return Ok(None);
        }
        let deck = repo.create_deck(name).map_err(|e| e.to_string())?;
        for (front, back, scenario, tags) in cards {
            let tag_strings = tags.into_iter().map(|s| s.to_string()).collect();
            repo.seed_insert_card_with_state(&deck.id, front, back, tag_strings, &scenario.build_state("", today))
                .map_err(|e| e.to_string())?;
        }
        Ok(Some(deck))
    }

    pub fn generate(repo: &Repository) -> Result<Vec<Deck>, String> {
        let today = Utc::now().date_naive();
        let existing_names: Vec<String> = repo
            .list_decks()
            .map_err(|e| e.to_string())?
            .iter()
            .map(|d| d.name.clone())
            .collect();
        let mut decks = Vec::new();

        // ── Deck 1: Deutsche Grammatik (15 Karten) ────────
        let cards1 = vec![
            ("der, die, das", "Bestimmte Artikel im Nominativ", Scenario::New, vec!["Artikel", "Grundlagen"]),
            ("Konjunktiv II", "würde + Infinitiv", Scenario::New, vec!["Verben", "Möglichkeit"]),
            ("Weil-Sätze", "Das finite Verb steht am Ende", Scenario::DueTodayFirst, vec!["Syntax", "Nebensätze"]),
            ("Adjektivdeklination", "Nach bestimmtem Artikel: der gute Mann", Scenario::DueTodayFirst, vec!["Adjektive", "Deklination"]),
            ("Präpositionen mit Dativ", "aus, bei, mit, nach, seit, von, zu", Scenario::DueTodayStruggling, vec!["Präpositionen", "Fälle"]),
            ("Plusquamperfekt", "hatte/war + Partizip II", Scenario::DueTomorrow, vec!["Verben", "Vergangenheit"]),
            ("Relativpronomen", "der, die, das (bezogen auf das Nomen)", Scenario::DueTomorrow, vec!["Pronomen", "Nebensätze"]),
            ("Trennbare Verben", "aufstehen → ich stehe auf", Scenario::DueIn7Days, vec!["Verben", "Syntax"]),
            ("Dass-Sätze", "Verb am Ende: Ich weiß, dass er kommt.", Scenario::DueIn7Days, vec!["Syntax", "Nebensätze"]),
            ("Präteritum (sein)", "ich war, du warst, er war", Scenario::Overdue, vec!["Verben", "Vergangenheit"]),
            ("Präteritum (haben)", "ich hatte, du hattest, er hatte", Scenario::Graduated, vec!["Verben", "Vergangenheit"]),
            ("Zukunft (Futur I)", "werden + Infinitiv: Ich werde gehen", Scenario::Graduated, vec!["Verben", "Zukunft"]),
            ("Komparativ", "Adjektiv + er: schnell -> schneller", Scenario::New, vec!["Adjektive", "Steigerung"]),
            ("Superlativ", "am + Adjektiv + sten: am schnellsten", Scenario::New, vec!["Adjektive", "Steigerung"]),
            ("Genitiv-Präpositionen", "wegen, während, trotz, anstatt", Scenario::DueTodayFirst, vec!["Präpositionen", "Fälle"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Grammatik Basis", cards1, today, &existing_names)? {
            decks.push(d);
        }

        // ── Deck 2: Weltgeschichte (8 Karten) ────────────
        let cards2 = vec![
            ("Zweiter Weltkrieg Ende", "1945", Scenario::New, vec!["20. Jahrhundert", "Krieg"]),
            ("Französische Revolution", "1789", Scenario::DueTodayFirst, vec!["Europa", "18. Jahrhundert"]),
            ("Kleopatra", "Letzte Pharaonin Ägyptens", Scenario::DueTodayFirst, vec!["Antike", "Ägypten"]),
            ("Magna Carta", "1215: Englische Verfassungsurkunde", Scenario::Overdue, vec!["Mittelalter", "England"]),
            ("Berliner Mauer Fall", "9. November 1989", Scenario::Graduated, vec!["20. Jahrhundert", "Deutschland"]),
            ("Industrielle Revolution", "Übergang zur Maschinenproduktion (~1760)", Scenario::DueIn7Days, vec!["Wirtschaft", "Europa"]),
            ("Entdeckung Amerikas", "1492 durch Christoph Kolumbus", Scenario::DueIn30Days, vec!["Entdecker", "15. Jahrhundert"]),
            ("Mondlandung", "1969 (Apollo 11)", Scenario::Graduated, vec!["20. Jahrhundert", "Raumfahrt"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Weltgeschichte Extra", cards2, today, &existing_names)? {
            decks.push(d);
        }

        // ── Deck 3: Biologie Grundlagen (3 Karten) ───────
        let cards3 = vec![
            ("Zellatmung", "C₆H₁₂O₆ + 6O₂ → 6CO₂ + 6H₂O + Energie", Scenario::New, vec!["Zelle", "Stoffwechsel"]),
            ("Photosynthese", "6CO₂ + 6H₂O + Licht → C₆H₁₂O₆ + 6O₂", Scenario::New, vec!["Pflanzen", "Stoffwechsel"]),
            ("DNA-Basen", "Adenin, Thymin, Guanin, Cytosin", Scenario::New, vec!["Genetik", "Zelle"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Biologie Intro", cards3, today, &existing_names)? {
            decks.push(d);
        }

        Ok(decks)
    }
}
