use crate::db::models::{CardState, Deck};
use crate::db::repository::Repository;
use chrono::{NaiveDate, Utc};
use std::collections::HashSet;

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
    fn build_deck(
        repo: &Repository,
        name: &str,
        cards: Vec<SeedCard>,
        today: NaiveDate,
        existing_decks: &[Deck],
    ) -> Result<Option<Deck>, String> {
        let (deck, created) = match existing_decks.iter().find(|deck| deck.name == name) {
            Some(deck) => (deck.clone(), false),
            None => (repo.create_deck(name).map_err(|e| e.to_string())?, true),
        };
        let mut existing_fronts: HashSet<String> = repo
            .list_cards(&deck.id)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|card| card.front)
            .collect();

        for card in cards {
            if existing_fronts.contains(card.front) {
                continue;
            }
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
            )
            .map_err(|e| e.to_string())?;
            existing_fronts.insert(card.front.to_string());
        }
        Ok(created.then_some(deck))
    }

    pub fn generate(repo: &Repository) -> Result<Vec<Deck>, String> {
        let today = Utc::now().date_naive();
        let mut existing_decks = repo.list_decks().map_err(|e| e.to_string())?;
        let current_logic_name = "Formale Logik & Mengenlehre";
        let legacy_logic_name = "Formale Logik & Problemlösen";
        let has_current_logic_deck = existing_decks
            .iter()
            .any(|deck| deck.name == current_logic_name);
        if !has_current_logic_deck {
            if let Some(legacy_deck) = existing_decks
                .iter_mut()
                .find(|deck| deck.name == legacy_logic_name)
            {
                repo.update_deck(&legacy_deck.id, current_logic_name)
                    .map_err(|e| e.to_string())?;
                legacy_deck.name = current_logic_name.to_string();
            }
        }
        let mut decks = Vec::new();

        // Helper macro/closure for basic cards
        let basic = |front: &'static str,
                     back: &'static str,
                     scenario: Scenario,
                     tags: Vec<&'static str>| SeedCard {
            front,
            back,
            scenario,
            tags,
            card_type: "basic",
            content: None,
            reasoning: None,
        };

        // ── Deck 1: Deutsche Grammatik (18 Karten) ────────
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
            SeedCard {
                front: "Welchen Kasus verlangt die Präposition **mit**?",
                back: "[x] Dativ\n[ ] Akkusativ\n[ ] Genitiv\n[ ] Nominativ",
                scenario: Scenario::New,
                tags: vec!["Präpositionen", "Fälle"],
                card_type: "multiple_choice",
                content: Some(r#"{"options":[{"text":"Dativ","correct":true},{"text":"Akkusativ","correct":false},{"text":"Genitiv","correct":false},{"text":"Nominativ","correct":false}]}"#),
                reasoning: Some("Die Präposition **mit** gehört zu den Präpositionen, die immer den Dativ verlangen."),
            },
            SeedCard {
                front: "In einem eingeleiteten Nebensatz steht das ==finite Verb am Ende==.",
                back: "Beispiel: Ich bleibe zu Hause, weil es heute regnet.",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Syntax", "Nebensätze"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Subjunktionen wie *weil*, *obwohl* oder *dass* leiten einen Nebensatz mit Verbendstellung ein."),
            },
            SeedCard {
                front: "Bringe die Satzglieder in eine natürliche Hauptsatz-Reihenfolge:",
                back: "1. Morgen\n2. werde\n3. ich\n4. in der Bibliothek\n5. lernen",
                scenario: Scenario::DueTomorrow,
                tags: vec!["Syntax", "Satzbau"],
                card_type: "ordering",
                content: Some(r#"{"items":["Morgen","werde","ich","in der Bibliothek","lernen"]}"#),
                reasoning: Some("Im deutschen Aussagesatz steht das finite Verb an zweiter Position; der infinite Verbteil bildet häufig die Satzklammer am Ende."),
            },
        ];
        if let Some(d) = Self::build_deck(repo, "Grammatik Basis", cards1, today, &existing_decks)?
        {
            decks.push(d);
        }

        // ── Deck 2: Weltgeschichte (11 Karten) ────────────
        let cards2 = vec![
            basic("Zweiter Weltkrieg Ende", "1945", Scenario::New, vec!["20. Jahrhundert", "Krieg"]),
            basic("Französische Revolution", "1789", Scenario::DueTodayFirst, vec!["Europa", "18. Jahrhundert"]),
            basic("Kleopatra", "Letzte Pharaonin Ägyptens", Scenario::DueTodayFirst, vec!["Antike", "Ägypten"]),
            basic("Magna Carta", "1215: Englische Verfassungsurkunde", Scenario::Overdue, vec!["Mittelalter", "England"]),
            basic("Berliner Mauer Fall", "9. November 1989", Scenario::Graduated, vec!["20. Jahrhundert", "Deutschland"]),
            basic("Industrielle Revolution", "Übergang zur Maschinenproduktion (~1760)", Scenario::DueIn7Days, vec!["Wirtschaft", "Europa"]),
            basic("Entdeckung Amerikas", "1492 durch Christoph Kolumbus", Scenario::DueIn30Days, vec!["Entdecker", "15. Jahrhundert"]),
            basic("Mondlandung", "1969 (Apollo 11)", Scenario::Graduated, vec!["20. Jahrhundert", "Raumfahrt"]),
            SeedCard {
                front: "Welches Ereignis fand chronologisch zuerst statt?",
                back: "[x] Unterzeichnung der Magna Carta\n[ ] Französische Revolution\n[ ] Fall der Berliner Mauer\n[ ] Mondlandung",
                scenario: Scenario::New,
                tags: vec!["Chronologie", "Epochen"],
                card_type: "multiple_choice",
                content: Some(r#"{"options":[{"text":"Unterzeichnung der Magna Carta","correct":true},{"text":"Französische Revolution","correct":false},{"text":"Fall der Berliner Mauer","correct":false},{"text":"Mondlandung","correct":false}]}"#),
                reasoning: Some("Die Magna Carta wurde 1215 besiegelt, deutlich vor 1789, 1969 und 1989."),
            },
            SeedCard {
                front: "Johannes Gutenberg revolutionierte im 15. Jahrhundert die Wissensverbreitung durch den ==Buchdruck mit beweglichen Lettern==.",
                back: "Seine Technik machte Bücher schneller und in größeren Auflagen produzierbar.",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Frühe Neuzeit", "Mediengeschichte"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Der mechanische Buchdruck senkte die Kosten der Vervielfältigung und beschleunigte die Verbreitung von Ideen."),
            },
            SeedCard {
                front: "Ordne diese Phasen der Französischen Revolution chronologisch:",
                back: "1. Einberufung der Generalstände\n2. Sturm auf die Bastille\n3. Ausrufung der Republik\n4. Herrschaft des Direktoriums",
                scenario: Scenario::DueIn7Days,
                tags: vec!["Frankreich", "Chronologie"],
                card_type: "ordering",
                content: Some(r#"{"items":["Einberufung der Generalstände","Sturm auf die Bastille","Ausrufung der Republik","Herrschaft des Direktoriums"]}"#),
                reasoning: Some("Die Abfolge führt von Mai 1789 über Juli 1789 und September 1792 bis zum Direktorium ab 1795."),
            },
        ];
        if let Some(d) =
            Self::build_deck(repo, "Weltgeschichte Extra", cards2, today, &existing_decks)?
        {
            decks.push(d);
        }

        // ── Deck 3: Biologie Grundlagen (6 Karten) ───────
        let cards3 = vec![
            basic("Zellatmung", "C₆H₁₂O₆ + 6O₂ → 6CO₂ + 6H₂O + Energie", Scenario::New, vec!["Zelle", "Stoffwechsel"]),
            basic("Photosynthese", "6CO₂ + 6H₂O + Licht → C₆H₁₂O₆ + 6O₂", Scenario::New, vec!["Pflanzen", "Stoffwechsel"]),
            basic("DNA-Basen", "Adenin, Thymin, Guanin, Cytosin", Scenario::New, vec!["Genetik", "Zelle"]),
            SeedCard {
                front: "Welches Zellorganell erzeugt den größten Teil des ATP bei der Zellatmung?",
                back: "[x] Mitochondrium\n[ ] Ribosom\n[ ] Golgi-Apparat\n[ ] Lysosom",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Zelle", "Stoffwechsel"],
                card_type: "multiple_choice",
                content: Some(r#"{"options":[{"text":"Mitochondrium","correct":true},{"text":"Ribosom","correct":false},{"text":"Golgi-Apparat","correct":false},{"text":"Lysosom","correct":false}]}"#),
                reasoning: Some("In der inneren Mitochondrienmembran erzeugt die oxidative Phosphorylierung den größten Anteil des ATP."),
            },
            SeedCard {
                front: "In der DNA paart ==Adenin mit Thymin== und ==Guanin mit Cytosin==.",
                back: "A-T bilden zwei, G-C drei Wasserstoffbrückenbindungen.",
                scenario: Scenario::New,
                tags: vec!["Genetik", "DNA"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Die komplementäre Basenpaarung ermöglicht die präzise Replikation der DNA."),
            },
            SeedCard {
                front: "Ordne die vereinfachten Schritte der Proteinbiosynthese:",
                back: "1. DNA-Abschnitt wird transkribiert\n2. mRNA verlässt den Zellkern\n3. Ribosom bindet an die mRNA\n4. Aminosäuren werden zur Polypeptidkette verknüpft",
                scenario: Scenario::DueTomorrow,
                tags: vec!["Genetik", "Proteinbiosynthese"],
                card_type: "ordering",
                content: Some(r#"{"items":["DNA-Abschnitt wird transkribiert","mRNA verlässt den Zellkern","Ribosom bindet an die mRNA","Aminosäuren werden zur Polypeptidkette verknüpft"]}"#),
                reasoning: Some("Auf die Transkription im Zellkern folgt die Translation der mRNA an den Ribosomen."),
            },
        ];
        if let Some(d) = Self::build_deck(repo, "Biologie Intro", cards3, today, &existing_decks)? {
            decks.push(d);
        }

        // ── Deck 4: Formale Logik & Mengenlehre (alle Kartentypen) ────────
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
            SeedCard {
                front: "Wie viele Elemente besitzt die Potenzmenge $\\mathcal{P}(A)$ einer Menge $A$ mit drei Elementen?",
                back: "[x] 8\n[ ] 3\n[ ] 6\n[ ] 9",
                scenario: Scenario::New,
                tags: vec!["Mengenlehre", "Potenzmenge"],
                card_type: "multiple_choice",
                content: Some(r#"{"options":[{"text":"8","correct":true},{"text":"3","correct":false},{"text":"6","correct":false},{"text":"9","correct":false}]}"#),
                reasoning: Some("Eine $n$-elementige Menge besitzt genau $2^n$ Teilmengen. Für $n=3$ gilt $2^3=8$."),
            },
            SeedCard {
                front: "Bringe die Schritte eines Beweises von $A \\subseteq B$ in die richtige Reihenfolge:",
                back: "1. Wähle ein beliebiges Element x aus A\n2. Nutze die definierenden Eigenschaften von A\n3. Zeige, dass x die Eigenschaften von B erfüllt\n4. Folgere x ∈ B und damit A ⊆ B",
                scenario: Scenario::DueTodayFirst,
                tags: vec!["Mengenlehre", "Beweise"],
                card_type: "ordering",
                content: Some(r#"{"items":["Wähle ein beliebiges Element x aus A","Nutze die definierenden Eigenschaften von A","Zeige, dass x die Eigenschaften von B erfüllt","Folgere x ∈ B und damit A ⊆ B"]}"#),
                reasoning: Some("Eine Teilmengenbeziehung wird elementweise bewiesen: Jedes beliebige Element von A muss auch Element von B sein."),
            },
            SeedCard {
                front: "Die symmetrische Differenz $A \\triangle B$ enthält alle Elemente, die ==in genau einer der beiden Mengen== liegen.",
                back: "$A \\triangle B = (A \\setminus B) \\cup (B \\setminus A)$",
                scenario: Scenario::DueIn7Days,
                tags: vec!["Mengenlehre", "Mengenoperationen"],
                card_type: "cloze",
                content: None,
                reasoning: Some("Elemente aus der Schnittmenge werden ausgeschlossen; übrig bleiben die Elemente, die nur A oder nur B angehören."),
            },
        ];

        if let Some(d) = Self::build_deck(repo, current_logic_name, cards4, today, &existing_decks)?
        {
            decks.push(d);
        }

        Ok(decks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;
    use std::collections::HashSet;

    #[test]
    fn sample_decks_cover_every_interactive_card_type() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run_migrations(&conn).unwrap();
        let repo = Repository::new(conn);

        let decks = SeedGenerator::generate(&repo).unwrap();
        assert_eq!(decks.len(), 4);
        assert!(decks
            .iter()
            .any(|deck| deck.name == "Formale Logik & Mengenlehre"));

        let expected_types: HashSet<&str> = ["basic", "cloze", "multiple_choice", "ordering"]
            .into_iter()
            .collect();

        for deck in decks {
            let cards = repo.list_cards(&deck.id).unwrap();
            let actual_types: HashSet<&str> =
                cards.iter().map(|card| card.card_type.as_str()).collect();
            assert_eq!(
                actual_types, expected_types,
                "Kartentypen fehlen in {}",
                deck.name
            );

            for card in cards
                .iter()
                .filter(|card| card.card_type == "multiple_choice" || card.card_type == "ordering")
            {
                let content = card
                    .content
                    .as_deref()
                    .expect("Interaktive Karte ohne Inhalt");
                let parsed: serde_json::Value =
                    serde_json::from_str(content).unwrap_or_else(|error| {
                        panic!("Ungültiger Karteninhalt in '{}': {error}", card.front)
                    });
                let field = if card.card_type == "multiple_choice" {
                    "options"
                } else {
                    "items"
                };
                assert!(
                    parsed[field]
                        .as_array()
                        .is_some_and(|items| items.len() >= 2),
                    "Interaktive Karte '{}' benötigt mindestens zwei Einträge",
                    card.front
                );
            }
        }

        let card_count_before: usize = repo
            .list_decks()
            .unwrap()
            .iter()
            .map(|deck| repo.list_cards(&deck.id).unwrap().len())
            .sum();
        assert!(SeedGenerator::generate(&repo).unwrap().is_empty());
        let card_count_after: usize = repo
            .list_decks()
            .unwrap()
            .iter()
            .map(|deck| repo.list_cards(&deck.id).unwrap().len())
            .sum();
        assert_eq!(card_count_after, card_count_before);
    }

    #[test]
    fn legacy_logic_deck_is_renamed_and_completed() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run_migrations(&conn).unwrap();
        let repo = Repository::new(conn);
        repo.create_deck("Formale Logik & Problemlösen").unwrap();

        let created_decks = SeedGenerator::generate(&repo).unwrap();
        assert_eq!(created_decks.len(), 3);

        let decks = repo.list_decks().unwrap();
        assert!(decks
            .iter()
            .any(|deck| deck.name == "Formale Logik & Mengenlehre"));
        assert!(!decks
            .iter()
            .any(|deck| deck.name == "Formale Logik & Problemlösen"));

        let logic_deck = decks
            .iter()
            .find(|deck| deck.name == "Formale Logik & Mengenlehre")
            .unwrap();
        assert_eq!(repo.list_cards(&logic_deck.id).unwrap().len(), 23);
    }
}
