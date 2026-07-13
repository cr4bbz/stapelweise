# Stapelweise v1.0 Product Manifest

> **Leitbild:** Verstehen statt bloßes Erinnern.

**Stapelweise** ist eine lokale, freie und datensouveräne Desktop-Anwendung für intelligentes Wissensmanagement und Prüfungsvorbereitung.

---

## 1. Produktziele & Kernprinzipien

1. **Local-First & Datensouveränität**
   - 100% offline nutzbar ohne Zwangskonto oder Cloud-Abhängigkeit.
   - Alle Daten liegen lokal in SQLite (`stapelweise.db`).
   - Vollständiger JSON-Backup-Export und konfliktfreies Pre-flight-Import-System.

2. **Verstehen statt Memorieren**
   - Mehrere Karteikartentypen (Basic, Cloze, Multiple-Choice, Free-Text, Ordering).
   - Inhaltsstarke Begründungsfelder (`reasoning`), LaTeX-Formelunterstützung via KaTeX, Markdown.
   - Eingebettete Medienverwaltung mit concisen Referenz-Tags (`media:img_...`).

3. **Reproduzierbare Prüfungs-Engine**
   - Prüfungs-Templates mit konfigurierbaren Decks, Tags, Fragetypen, Zeitlimits und Bestehensschwellen.
   - Deterministische Fragensortierung via PRNG-Seed (`rand::SeedableRng`).
   - Unveränderliche Frage-Snapshots (`exam_questions`) für nachvollziehbare Audits und detailreiche Leistungsauswertungen nach Deck, Tag und Fragetyp.

4. **Gehärtetes Spaced Repetition (SM-2)**
   - Deterministische Intervall- und Ease-Factor-Berechnung.
   - Beidseitiges Clamping des Ease Factors (`1.3` bis `5.0`).
   - Vollständiger Review-Verlauf und unbegrenztes Undo für Bewertungen.

5. **Erweiterbarkeit & Interoperabilität**
   - Bi-direktionale Obsidian-Vault-Synchronisation für nahtloses Wissensmanagement.
   - Model Context Protocol (MCP) Servermodus für KI-gestütztes Lernen und Karteikartengenerierung.

---

## 2. Unterstützte Karteikartentypen

| Typ | Beschreibung | Speicherung |
|---|---|---|
| **`basic`** | Klassische Vorder- und Rückseite | Standard Markdown / KaTeX |
| **`cloze`** | Lückentext mit `==lücke==` oder `{{c1::lücke}}` | Extraktion von Platzhaltern |
| **`multiple_choice`** | Multiple Choice mit mehreren Antworten | JSON (`{ options: [...] }`) |
| **`free_text`** | Freitextantwort mit automatischem Abgleich | Standard Markdown / Text |
| **`ordering`** | Reihenfolge-Karten für Abläufe & Prozesse | JSON (`{ items: [...] }`) |

---

## 3. Fehler-Taxonomie (`ErrorCode`)

Stapelweise nutzt strukturierte Fehler-Payloads (`CommandErrorPayload`) über Tauri-IPC:

- `VALIDATION_FAILED` (400): Eingabefehler oder ungültige Kartendaten.
- `NOT_FOUND` (404): Karte, Deck oder Prüfung nicht gefunden.
- `DB_ERROR` (500): SQLite-Datenbankfehler.
- `IMPORT_INVALID_JSON` (400): Fehlerhaftes JSON im Backup- / Deck-Import.
- `IMPORT_FAILED` (422): Abbruch bei der Wiederherstellung / Validierung.
- `EXPORT_FAILED` (500): Fehler beim Erstellen der Backup-Datei.
- `OBSIDIAN_SYNC_ERROR` (500): Vault-Pfad ungültig oder Lese/Schreibfehler.
- `INTERNAL_ERROR` (500): Unerwarteter interner Systemfehler.

---

## 4. Systemanforderungen & Technologie-Stack

- **Desktop Framework:** Tauri 2 (Rust Backend, SvelteKit Frontend)
- **Frontend Stack:** Svelte 5 (Runes), TailwindCSS, TypeScript
- **Datenbank:** SQLite via `rusqlite`
- **Fenster-Modus:** 16:9 Aspect Ratio (1280x720, universelle Scrollbarkeit)
- **Unterstützte Betriebssysteme:** Windows 10/11, macOS, Linux
