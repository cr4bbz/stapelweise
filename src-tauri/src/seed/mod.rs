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
    pub fn generate(repo: &Repository) -> Result<Vec<Deck>, String> {
        let today = Utc::now().date_naive();
        let mut decks = Vec::new();

        // ── Deck 1: Deutsche Grammatik (8 Karten) ────────
        let deck1 = repo.create_deck("Deutsche Grammatik").map_err(|e| e.to_string())?;
        let cards1: Vec<(&str, &str, Scenario)> = vec![
            ("der, die, das", "Bestimmte Artikel im Nominativ: der (m), die (f), das (n)", Scenario::New),
            ("Konjunktiv II", "würde + Infinitiv: Ich würde gehen. Bei Hilfsverben: ich hätte, ich wäre", Scenario::New),
            ("Weil-Sätze", "Das finite Verb steht am Ende: ..., weil ich müde bin.", Scenario::DueTodayFirst),
            ("Adjektivdeklination", "Nach bestimmtem Artikel: der gute Mann, die schöne Frau, das kleine Kind", Scenario::DueTodayFirst),
            ("Präpositionen mit Dativ", "aus, bei, mit, nach, seit, von, zu — immer Dativ!", Scenario::DueTodayStruggling),
            ("Plusquamperfekt", "hatte/war + Partizip II: Ich hatte gegessen, ich war gegangen", Scenario::DueTomorrow),
            ("Relativpronomen", "der, die, das (bezogen auf das Nomen): Der Mann, der dort steht...", Scenario::DueTomorrow),
            ("Trennbare Verben", "aufstehen → ich stehe auf, anrufen → ich rufe an", Scenario::DueIn7Days),
        ];
        for (front, back, scenario) in cards1 {
            repo.seed_insert_card_with_state(&deck1.id, front, back, &scenario.build_state("", today))
                .map_err(|e| e.to_string())?;
        }
        decks.push(deck1);

        // ── Deck 2: Weltgeschichte (6 Karten) ────────────
        let deck2 = repo.create_deck("Weltgeschichte").map_err(|e| e.to_string())?;
        let cards2: Vec<(&str, &str, Scenario)> = vec![
            ("Wann endete der Zweite Weltkrieg?", "1945 — mit der bedingungslosen Kapitulation Deutschlands am 8. Mai", Scenario::New),
            ("Was war die Französische Revolution?", "1789: Sturz der Monarchie, Erklärung der Menschenrechte, Ende des Ancien Régime", Scenario::DueTodayFirst),
            ("Wer war Kleopatra?", "Letzte Pharaonin Ägyptens (69–30 v.Chr.), verband sich mit Caesar und Marcus Antonius", Scenario::DueTodayFirst),
            ("Was ist die Magna Carta?", "1215: Englische Verfassungsurkunde, schränkte königliche Macht ein, Grundlage des Rechtsstaats", Scenario::Overdue),
            ("Wann fiel die Berliner Mauer?", "9. November 1989 — Ende der deutschen Teilung, Symbol des Endes des Kalten Krieges", Scenario::Graduated),
            ("Was war die industrielle Revolution?", "Übergang von Handarbeit zur Maschinenproduktion, ~1760–1840, begann in England", Scenario::DueIn7Days),
        ];
        for (front, back, scenario) in cards2 {
            repo.seed_insert_card_with_state(&deck2.id, front, back, &scenario.build_state("", today))
                .map_err(|e| e.to_string())?;
        }
        decks.push(deck2);

        // ── Deck 3: Biologie Grundlagen (4 Karten, alle neu) ───────
        let deck3 = repo.create_deck("Biologie Grundlagen").map_err(|e| e.to_string())?;
        let cards3: Vec<(&str, &str, Scenario)> = vec![
            ("Zellatmung", "C₆H₁₂O₆ + 6O₂ → 6CO₂ + 6H₂O + Energie (ATP)", Scenario::New),
            ("Photosynthese", "6CO₂ + 6H₂O + Licht → C₆H₁₂O₆ + 6O₂", Scenario::New),
            ("DNA-Basenpaarung", "Adenin-Thymin (A-T), Guanin-Cytosin (G-C)", Scenario::New),
            ("Mitose vs. Meiose", "Mitose: 2 identische Tochterzellen (2n). Meiose: 4 Keimzellen mit halbem Satz (n).", Scenario::New),
        ];
        for (front, back, scenario) in cards3 {
            repo.seed_insert_card_with_state(&deck3.id, front, back, &scenario.build_state("", today))
                .map_err(|e| e.to_string())?;
        }
        decks.push(deck3);

        Ok(decks)
    }
}
