use crate::db::models::{CardState, Deck};
use crate::db::repository::Repository;
use chrono::{NaiveDate, Utc};

/// SM-2 scenario variants for seed data
pub(crate) enum Scenario {
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
                repetitions: 0,
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
                repetitions: 2,
                next_review: fmt(today + chrono::Duration::days(7)),
                total_reviews: 2,
                correct_streak: 2,
                last_review: Some(fmt(today - chrono::Duration::days(7))),
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
                interval: 30,
                ease_factor: 2.5,
                repetitions: 5,
                next_review: fmt(today - chrono::Duration::days(3)),
                total_reviews: 6,
                correct_streak: 5,
                last_review: Some(fmt(today - chrono::Duration::days(33))),
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

pub struct SeedCard<'a> {
    pub front: &'a str,
    pub back: &'a str,
    pub scenario: Scenario,
    pub tags: Vec<&'a str>,
    pub card_type: &'a str,
    pub content: Option<&'a str>,
    pub reasoning: Option<&'a str>,
}

impl SeedGenerator {
    fn build_deck(repo: &Repository, name: &str, cards: Vec<SeedCard>, today: NaiveDate, skip: &[String]) -> Result<Option<Deck>, String> {
        if skip.contains(&name.to_string()) {
            return Ok(None);
        }
        let deck = repo.create_deck(name).map_err(|e| e.to_string())?;
        for card in cards {
            let tag_strings = card.tags.into_iter().map(|s| s.to_string()).collect();
            repo.seed_insert_card_with_state(
                &deck.id,
                card.front,
                card.back,
                tag_strings,
                &card.scenario.build_state("", today),
                card.card_type,
                card.content,
                card.reasoning,
            ).map_err(|e| e.to_string())?;
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

        // Helper macro/closure for basic cards
        let basic = |front: &'static str, back: &'static str, scenario: Scenario, tags: Vec<&'static str>| SeedCard {
            front, back, scenario, tags, card_type: "basic", content: None, reasoning: None,
        };

        // ── Deck 1: Deutsche Grammatik (15 Karten) ────────
        let cards1 = vec![
            basic("der, die, das", "Bestimmte Artikel im Nominativ", Scenario::New, vec!["Artikel", "Grundlagen"]),
            basic("Konjunktiv II", "würde + Infinitiv", Scenario::New, vec!["Verben", "Möglichkeit"]),
            basic("Weil-Sätze", "Das finite Verb steht am Ende", Scenario::DueTodayFirst, vec!["Syntax", "Nebensätze"]),
            basic("Adjektivdeklination", "Nach bestimmtem Artikel: der gute Mann", Scenario::DueTodayFirst, vec!["Adjektive", "Deklination"]),
            basic("Präpositionen mit Dativ", "aus, bei, mit, nach, seit, von, zu", Scenario::DueTodayStruggling, vec!["Präpositionen", "Fälle"]),
            basic("Plusquamperfekt", "hatte/war + Partizip II", Scenario::DueTomorrow, vec!["Verben", "Vergangenheit"]),
            basic("Relativpronomen", "der, die, das (bezogen auf das Nomen)", Scenario::DueTomorrow, vec!["Pronomen", "Nebensätze"]),
            basic("Trennbare Verben", "aufstehen → ich stehe auf", Scenario::DueIn7Days, vec!["Verben", "Syntax"]),
            basic("Dass-Sätze", "Verb am Ende: Ich weiß, dass er kommt.", Scenario::DueIn7Days, vec!["Syntax", "Nebensätze"]),
            basic("Präteritum (sein)", "ich war, du warst, er war", Scenario::Overdue, vec!["Verben", "Vergangenheit"]),
            basic("Präteritum (haben)", "ich hatte, du hattest, er hatte", Scenario::Graduated, vec!["Verben", "Vergangenheit"]),
            basic("Zukunft (Futur I)", "werden + Infinitiv: Ich werde gehen", Scenario::Graduated, vec!["Verben", "Zukunft"]),
            basic("Komparativ", "Adjektiv + er: schnell -> schneller", Scenario::New, vec!["Adjektive", "Steigerung"]),
            basic("Superlativ", "am + Adjektiv + sten: am schnellsten", Scenario::New, vec!["Adjektive", "Steigerung"]),
            basic("Genitiv-Präpositionen", "wegen, während, trotz, anstatt", Scenario::DueTodayFirst, vec!["Präpositionen", "Fälle"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Grammatik Basis", cards1, today, &existing_names)? {
            decks.push(d);
        }

        // ── Deck 2: Weltgeschichte (8 Karten) ────────────
        let cards2 = vec![
            basic("Zweiter Weltkrieg Ende", "1945", Scenario::New, vec!["20. Jahrhundert", "Krieg"]),
            basic("Französische Revolution", "1789", Scenario::DueTodayFirst, vec!["Europa", "18. Jahrhundert"]),
            basic("Kleopatra", "Letzte Pharaonin Ägyptens", Scenario::DueTodayFirst, vec!["Antike", "Ägypten"]),
            basic("Magna Carta", "1215: Englische Verfassungsurkunde", Scenario::Overdue, vec!["Mittelalter", "England"]),
            basic("Berliner Mauer Fall", "9. November 1989", Scenario::Graduated, vec!["20. Jahrhundert", "Deutschland"]),
            basic("Industrielle Revolution", "Übergang zur Maschinenproduktion (~1760)", Scenario::DueIn7Days, vec!["Wirtschaft", "Europa"]),
            basic("Entdeckung Amerikas", "1492 durch Christoph Kolumbus", Scenario::DueIn30Days, vec!["Entdecker", "15. Jahrhundert"]),
            basic("Mondlandung", "1969 (Apollo 11)", Scenario::Graduated, vec!["20. Jahrhundert", "Raumfahrt"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Weltgeschichte Extra", cards2, today, &existing_names)? {
            decks.push(d);
        }

        // ── Deck 3: Biologie Grundlagen (3 Karten) ───────
        let cards3 = vec![
            basic("Zellatmung", "C₆H₁₂O₆ + 6O₂ → 6CO₂ + 6H₂O + Energie", Scenario::New, vec!["Zelle", "Stoffwechsel"]),
            basic("Photosynthese", "6CO₂ + 6H₂O + Licht → C₆H₁₂O₆ + 6O₂", Scenario::New, vec!["Pflanzen", "Stoffwechsel"]),
            basic("DNA-Basen", "Adenin, Thymin, Guanin, Cytosin", Scenario::New, vec!["Genetik", "Zelle"]),
        ];
        if let Some(d) = Self::build_deck(repo, "Biologie Intro", cards3, today, &existing_names)? {
            decks.push(d);
        }

        // ── Deck 4: Formale Logik & Problemlösen (20 Karten mit ALLEN Kartentypen) ────────
        let cards4 = vec![
            // 1. MC De Morgan
            SeedCard {
                front: "Welche der folgenden Aussagen ist äquivalent zu $\\neg (P \\land Q)$ nach den De Morganschen Gesetzen?",
                back: "[x] \\neg P \\lor \\neg Q\n[ ] \\neg P \\land \\neg Q\n[ ] P \\lor Q\n[ ] \\neg (P \\lor Q)",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Aussagenlogik", "Äquivalenzen"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "\\neg P \\lor \\neg Q", "correct": true}, {"text": "\\neg P \\land \\neg Q", "correct": false}, {"text": "P \\lor Q", "correct": false}, {"text": "\\neg (P \\lor Q)", "correct": false}]}"#),
                reasoning: Some("Nach den De Morganschen Gesetzen wird die Negation einer Konjunktion $\\neg(P \\land Q)$ zu einer Disjunktion der Negationen $\\neg P \\lor \\neg Q$."),
            },
            // 2. MC XOR
            SeedCard {
                front: "Wann ist die exklusive Disjunktion (XOR / $\\oplus$) zweier Aussagen $P$ und $Q$ wahr?",
                back: "[x] Genau dann, wenn exakt eine der beiden Aussagen wahr ist\n[ ] Wenn beide Aussagen wahr sind\n[ ] Wenn beide Aussagen falsch sind",
                scenario: Scenario::New,
                tags: vec!["Schaltalgebra", "Operatoren"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "Genau dann, wenn exakt eine der beiden Aussagen wahr ist", "correct": true}, {"text": "Wenn beide Aussagen wahr sind", "correct": false}, {"text": "Wenn beide Aussagen falsch sind", "correct": false}, {"text": "Wenn mindestens eine Aussage wahr ist", "correct": false}]}"#),
                reasoning: Some("XOR liefert 1, wenn sich die Wahrheitswerte unterscheiden (1,0 oder 0,1). Wenn beide 1 sind, liefert OR 1, aber XOR 0."),
            },
            // 3. MC Fehlschlüsse
            SeedCard {
                front: "Welchem logischen Fehlschluss entspricht das Schema: 'Person A behauptet X. Person B greift den Charakter von A an, also ist X falsch'?",
                back: "[x] Argumentum ad Hominem\n[ ] Strohmann-Argument (Straw Man)\n[ ] Falsches Dilemma",
                scenario: Scenario::DueTomorrow,
                tags: vec!["Argumentation", "Fehlschlüsse"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "Argumentum ad Hominem", "correct": true}, {"text": "Strohmann-Argument (Straw Man)", "correct": false}, {"text": "Falsches Dilemma (False Dilemma)", "correct": false}, {"text": "Post hoc ergo propter hoc", "correct": false}]}"#),
                reasoning: Some("Beim Ad Hominem wird die Person anstelle ihres sachlichen Arguments angegriffen."),
            },
            // 4. MC Implikation
            SeedCard {
                front: "Welcher Ausdruck ist logisch äquivalent zur Implikation $P \\Rightarrow Q$?",
                back: "[x] \\neg P \\lor Q\n[x] \\neg Q \\Rightarrow \\neg P (Kontraposition)\n[ ] P \\land \\neg Q",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Aussagenlogik", "Implikation"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "\\neg P \\lor Q", "correct": true}, {"text": "\\neg Q \\Rightarrow \\neg P (Kontraposition)", "correct": true}, {"text": "P \\land \\neg Q", "correct": false}, {"text": "Q \\Rightarrow P", "correct": false}]}"#),
                reasoning: Some("Die Implikation $P \\Rightarrow Q$ ist äquivalent zu $\\neg P \\lor Q$ sowie ihrer Kontraposition $\\neg Q \\Rightarrow \\neg P$."),
            },
            // 5. MC Tautologie
            SeedCard {
                front: "Welche der folgenden Aussagen ist eine Tautologie (stets wahr)?",
                back: "[x] P \\lor \\neg P\n[ ] P \\land \\neg P\n[ ] P \\Rightarrow \\neg P",
                scenario: Scenario::Overdue,
                tags: vec!["Tautologie", "Aussagenlogik"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "P \\lor \\neg P", "correct": true}, {"text": "P \\land \\neg P", "correct": false}, {"text": "P \\Rightarrow \\neg P", "correct": false}, {"text": "P \\Leftrightarrow \\neg P", "correct": false}]}"#),
                reasoning: Some("Das Gesetz vom ausgeschlossenen Dritten (Tertium non datur) $P \\lor \\neg P$ ist immer wahr."),
            },
            // 6. MC Quantoren
            SeedCard {
                front: "Wie wird die Negation der Existenzaussage $\\neg (\\exists x : P(x))$ ausgedrückt?",
                back: "[x] \\forall x : \\neg P(x)\n[ ] \\exists x : \\neg P(x)\n[ ] \\neg (\\forall x : P(x))",
                scenario: Scenario::Graduated,
                tags: vec!["Prädikatenlogik", "Quantoren"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "\\forall x : \\neg P(x)", "correct": true}, {"text": "\\exists x : \\neg P(x)", "correct": false}, {"text": "\\neg (\\forall x : P(x))", "correct": false}, {"text": "\\forall x : P(x)", "correct": false}]}"#),
                reasoning: Some("Wenn nicht existiert ein x für das P(x) gilt, gilt für ALLE x, dass P(x) nicht zutrifft."),
            },
            // 7. MC Mengenlehre
            SeedCard {
                front: "Gegeben sind die Mengen $A = \\{1, 2, 3\\}$ und $B = \\{3, 4, 5\\}$. Was ist die Schnittmenge $A \\cap B$?",
                back: "[x] \\{3\\}\n[ ] \\{1, 2, 3, 4, 5\\}\n[ ] \\emptyset",
                scenario: Scenario::New,
                tags: vec!["Mengenlehre", "Grundlagen"],
                card_type: "multiple_choice",
                content: Some(r#"{"options": [{"text": "\\{3\\}", "correct": true}, {"text": "\\{1, 2, 3, 4, 5\\}", "correct": false}, {"text": "\\{1, 2, 4, 5\\}", "correct": false}, {"text": "\\emptyset", "correct": false}]}"#),
                reasoning: Some("Die Schnittmenge enthält alle Elemente, die sowohl in A als auch in B enthalten sind. Nur die 3 liegt in beiden Mengen."),
            },
            // 8. Sequenz Beweis
            SeedCard {
                front: "Bringe die Schritte eines direkten mathematischen Beweises in die korrekte logische Reihenfolge:",
                back: "1. Voraussetzung A annehmen\n2. Gelten von bekannten Axiomen & Theoremen anwenden\n3. Zwischenbehauptung herleiten\n4. Zielbehauptung B etablieren (Q.E.D.)",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Beweise", "Methodik"],
                card_type: "ordering",
                content: Some(r#"{"items": ["Voraussetzung A annehmen", "Gelten von bekannten Axiomen & Theoremen anwenden", "Zwischenbehauptung herleiten", "Zielbehauptung B etablieren (Q.E.D.)"]}"#),
                reasoning: Some("Ein direkter Beweis beginnt bei den Prämissen und leitet schrittweise durch logische Schlussregeln die Konklusion her."),
            },
            // 9. Sequenz Operatorenrangfolge
            SeedCard {
                front: "Bringe die logischen Operatoren in die korrekte Auswertungsreihenfolge (von HÖCHSTER Bindungsstärke zu NIEDRIGSTER):",
                back: "1. Negation (NOT / ¬)\n2. Konjunktion (AND / ∧)\n3. Disjunktion (OR / ∨)\n4. Implikation (⇒)\n5. Äquivalenz (⇔)",
                scenario: Scenario::New,
                tags: vec!["Syntax", "Operatoren"],
                card_type: "ordering",
                content: Some(r#"{"items": ["Negation (NOT / ¬)", "Konjunktion (AND / ∧)", "Disjunktion (OR / ∨)", "Implikation (⇒)", "Äquivalenz (⇔)"]}"#),
                reasoning: Some("Klammern bilden die Ausnahme, ansonsten bindet NOT am stärksten, gefolgt von AND, OR, Implikation und schließlich Äquivalenz."),
            },
            // 10. Sequenz Wissenschaftliche Methode
            SeedCard {
                front: "Bringe die Phasen der wissenschaftlichen Erkenntnisgewinnung in die logische Prozesskette:",
                back: "1. Beobachtung eines Phänomens\n2. Aufstellen einer überprüfbaren Hypothese\n3. Durchführung eines kontrollierten Experiments\n4. Analyse der Daten & statistische Auswertung\n5. Bestätigung oder Verwerfung der Hypothese",
                scenario: Scenario::DueIn7Days,
                tags: vec!["Wissenschaftstheorie", "Methodik"],
                card_type: "ordering",
                content: Some(r#"{"items": ["Beobachtung eines Phänomens", "Aufstellen einer überprüfbaren Hypothese", "Durchführung eines kontrollierten Experiments", "Analyse der Daten & statistische Auswertung", "Bestätigung oder Verwerfung der Hypothese"]}"#),
                reasoning: Some("Erkenntnis entspringt der empirischen Beobachtung und wird durch gezieltes Testen falsifiziert oder verifiziert."),
            },
            // 11. Sequenz Binary Search
            SeedCard {
                front: "Ordne die Ausführungsschritte des Algorithmus Binäre Suche (Binary Search) für ein sortiertes Feld:",
                back: "1. Mitte des aktuellen Intervalls bestimmen\n2. Vergleich des Mittelelements mit dem Gesuchten\n3. Bei Treffer: Index zurückgeben\n4. Bei kleinerem Wert: Rechtes Teilintervall wählen\n5. Bei größerem Wert: Linkes Teilintervall wählen",
                scenario: Scenario::Graduated,
                tags: vec!["Informatik", "Algorithmen"],
                card_type: "ordering",
                content: Some(r#"{"items": ["Mitte des aktuellen Intervalls bestimmen", "Vergleich des Mittelelements mit dem Gesuchten", "Bei Treffer: Index zurückgeben", "Bei kleinerem Wert: Rechtes Teilintervall wählen", "Bei größerem Wert: Linkes Teilintervall wählen"]}"#),
                reasoning: Some("Binäre Suche halbiert in jedem Schritt den Suchraum $O(\\log n)$."),
            },
            // 12. Sequenz IDEAL Framework
            SeedCard {
                front: "Sortiere die Phasen des IDEAL Problem-Solving Frameworks in die richtige Reihenfolge:",
                back: "1. Identify: Problem identifizieren\n2. Define: Ziel und Rahmenbedingungen definieren\n3. Explore: Mögliche Strategien erkunden\n4. Act: Gewählte Strategie umsetzen\n5. Look: Ergebnisse reflektieren & bewerten",
                scenario: Scenario::New,
                tags: vec!["Problemlösen", "Kognition"],
                card_type: "ordering",
                content: Some(r#"{"items": ["Identify: Problem identifizieren", "Define: Ziel und Rahmenbedingungen definieren", "Explore: Mögliche Strategien erkunden", "Act: Gewählte Strategie umsetzen", "Look: Ergebnisse reflektieren & bewerten"]}"#),
                reasoning: Some("IDEAL steht für Identify, Define, Explore, Act, Look back."),
            },
            // 13. Cloze Modus Ponens
            SeedCard {
                front: "Der ==Modus Ponens== lautet: Wenn aus $P$ die Aussage $Q$ folgt ($P \\Rightarrow Q$) und $P$ wahr ist, dann ist ==$Q$ wahr==.",
                back: "Der Modus Ponens lautet: Wenn aus $P$ die Aussage $Q$ folgt ($P \\Rightarrow Q$) und $P$ wahr ist, dann ist $Q$ wahr.",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Schlussregeln", "Aussagenlogik"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Modus Ponens ist die grundlegendste Abtrennungsregel der klassischen Logik."),
            },
            // 14. Cloze Modus Tollens
            SeedCard {
                front: "Der ==Modus Tollens== lautet: Wenn $P \\Rightarrow Q$ gilt und $Q$ falsch ist ($\\neg Q$), dann muss auch ==$P$ falsch sein ($\\neg P$)==.",
                back: "Der Modus Tollens lautet: Wenn $P \\Rightarrow Q$ gilt und $Q$ falsch ist ($\\neg Q$), dann muss auch $P$ falsch sein ($\\neg P$).",
                scenario: Scenario::DueTomorrow,
                tags: vec!["Schlussregeln", "Aussagenlogik"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Der Modus Tollens schließt von der Falschheit der Konsequenz auf die Falschheit der Prämisse."),
            },
            // 15. Cloze Kontradiktion
            SeedCard {
                front: "Eine Aussageformel, die unter jeder Belegung mit Wahrheitswerten falsch evaluiert, nennt man eine ==Kontradiktion (Widerspruch)==.",
                back: "Eine Aussageformel, die unter jeder Belegung mit Wahrheitswerten falsch evaluiert, nennt man eine Kontradiktion (Widerspruch).",
                scenario: Scenario::DueIn30Days,
                tags: vec!["Aussagenlogik", "Begriffe"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Das Gegenteil einer Kontradiktion ist eine Tautologie."),
            },
            // 16. Cloze Gödel
            SeedCard {
                front: "Gödels erster Unvollständigkeitssatz besagt, dass jedes hinreichend mächtige widerspruchsfreie formale System ==unvollständig== ist, d.h. es gibt Aussagen, die weder ==bewiesen noch widerlegt== werden können.",
                back: "Gödels erster Unvollständigkeitssatz besagt, dass jedes hinreichend mächtige widerspruchsfreie formale System unvollständig ist, d.h. es gibt Aussagen, die weder bewiesen noch widerlegt werden können.",
                scenario: Scenario::Overdue,
                tags: vec!["Metamathematik", "Gödel"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Kurt Gödel erschütterte 1931 damit das Hilbertsche Programm der Mathematik."),
            },
            // 17. Basic Implikation Table
            SeedCard {
                front: "Wann genau ist die logische Implikation $P \\Rightarrow Q$ FALSCH?",
                back: "Ausschließlich dann, wenn **$P$ wahr** ist, aber **$Q$ falsch** ist ($1 \\Rightarrow 0$).",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Wahrheitstafeln", "Implikation"],
                card_type: "basic",
                content: None,
                reasoning: Some("Eine Implikation ist aus Falschem (Ex falso quodlibet) immer wahr. Sie scheitert nur, wenn aus Wahrem Falsches gefolgert werden soll."),
            },
            // 18. Basic SAT Problem
            SeedCard {
                front: "Was versteht man unter dem **Boolean Satisfiability Problem (SAT)** in der theoretischen Informatik?",
                back: "Die Frage, ob es für eine gegebene boolesche Formel mindestens eine Wahrheitswertbelegung der Variablen gibt, sodass die Formel **wahr ($1$)** wird.",
                scenario: Scenario::New,
                tags: vec!["Komplexitätstheorie", "SAT"],
                card_type: "basic",
                content: None,
                reasoning: Some("SAT war das allererste Problem, für das Stephen Cook 1971 die NP-Vollständigkeit bewies."),
            },
            // 19. Basic Reductio ad Absurdum
            SeedCard {
                front: "Wie funktioniert das Beweisprinzip **Reductio ad Absurdum** (Widerspruchsbeweis)?",
                back: "Man nimmt das **Gegenteil (die Negation $\\neg A$)** der zu beweisenden Aussage an und leitet einen **logischen Widerspruch** ($B \\land \\neg B$) her.",
                scenario: Scenario::DueTomorrow,
                tags: vec!["Beweisverfahren", "Logik"],
                card_type: "basic",
                content: None,
                reasoning: Some("Da im zweiwertigen System eine Aussage nur wahr oder falsch sein kann, beweist die Unmöglichkeit der Negation das Original."),
            },
            // 20. Basic Distributivgesetze
            SeedCard {
                front: "Formuliere die beiden **Distributivgesetze** für die Verknüpfung von Konjunktion ($\\land$) und Disjunktion ($\\lor$).",
                back: "1. $P \\land (Q \\lor R) \\equiv (P \\land Q) \\lor (P \\land R)$\n2. $P \\lor (Q \\land R) \\equiv (P \\lor Q) \\land (P \\lor R)$",
                scenario: Scenario::Graduated,
                tags: vec!["Äquivalenzen", "Algebra"],
                card_type: "basic",
                content: None,
                reasoning: Some("Im Gegensatz zur gewohnten Arithmetik verteilt sich in der Booleschen Logik sowohl die Konjunktion über die Disjunktion als auch die Disjunktion über die Konjunktion!"),
            },
        ];

        if let Some(d) = Self::build_deck(repo, "Formale Logik & Problemlösen", cards4, today, &existing_names)? {
            decks.push(d);
        }

        Ok(decks)
    }
}
