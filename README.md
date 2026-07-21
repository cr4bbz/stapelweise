# Stapelweise

Eine lokale Desktop-App zum Erstellen, Organisieren und Wiederholen von Karteikarten. Stapelweise verbindet einen anpassbaren Lernplan nach SM-2 mit einem modularen Dashboard, Rich-Content-Karten und Arbeitsablaeufen fuer Studium und Selbstlernen.

Die App speichert ihre Daten lokal in SQLite. Es gibt keinen Account-Zwang und die Kernfunktionen arbeiten offline.

## Highlights

- **Lernen mit SM-2:** Konfigurierbare Sessions, Wiederholungslogik, Bestehensgrenze, Fortschritt und Rueckgaengig per `Ctrl/Cmd+Z`.
- **Unterbrechungsfrei lernen:** Angefangene Lern- und freie Uebungsrunden bleiben beim Verlassen der Ansicht erhalten und koennen fortgesetzt werden.
- **Mehr als Vorderseite/Rueckseite:** Basis-, Luecken-, Multiple-Choice-, Freitext- und Reihenfolge-Karten.
- **Rich Content:** Markdown, LaTex mit KaTeX, Bilder, Begruendungen, Tags und optionale Sprachangaben fuer Vorder- und Rueckseite.
- **Eigenes Dashboard:** Module lassen sich verschieben, in festen responsiven Groessen anordnen, ausblenden und wieder hinzufuegen. Dazu gehoeren einzelne Stapel, Pruefungen, Lernlage, Tags, Timer, Meilensteine, Suchfeld, Einstellungen, Einzelkarten und Spacer.
- **Organisation:** Globale Suche, Archive fuer Stapel und Pruefungen, Tags, Kartenstatistiken und Beispielinhalte fuer alle Kartentypen.
- **Pruefungen:** Pruefungen mit Datum und relevanten Stapeln planen, gezielt lernen oder eine Simulation starten.
- **Druckansicht:** Karteikarten als A4-Studentenlernblatt ausgeben. Vorder- und Rueckseite stehen passend nebeneinander.
- **Lokalisierung und Darstellung:** Deutsch, Englisch, Spanisch, Franzoesisch und Portugiesisch sowie Light-, Dark- und abgestimmte Farbthemen.

## Karten und Lernen

### Kartentypen

| Typ | Einsatz |
| --- | --- |
| Basis | Klassische Frage und Antwort |
| Luecke | Cloze-Karten fuer Begriffe in einem Text |
| Multiple Choice | Auswahl mit einer oder mehreren Lernoptionen |
| Freitext | Antwort selbst formulieren und anschliessend bewerten |
| Reihenfolge | Schritte oder Begriffe in die richtige Reihenfolge bringen |

Alle Kartentypen koennen Markdown und LaTex enthalten. Verwende `$...$` fuer Inline-Mathematik und `$$...$$` fuer abgesetzte Formeln. Bilder lassen sich direkt im Editor einfuegen; externe Medien werden lokal verwaltet.

### Lernrunden

- Eine Lernrunde kann einen Stapel, mehrere Stapel, Tags oder eine Auswahl von Karten verwenden.
- Freie Uebungen funktionieren auch dann, wenn keine Karte faellig ist und veraendern den Lernplan nicht.
- Die Session-Groesse, die Bestehensgrenze und der initiale Ease-Faktor sind in den Einstellungen konfigurierbar.
- Falsch bewertete Karten kehren innerhalb der Runde wieder. Die Bewertungsleiste zeigt sichtbar, welche Bewertung als bestanden gilt.
- `Leertaste` deckt die Karte auf, `1` bis `4` bewertet sie, `Ctrl/Cmd+Z` nimmt die letzte Bewertung zurueck und `Esc` verlaesst die Ansicht. Mit `?` erscheint die vollstaendige Tastaturhilfe.

## Dashboard und Bibliothek

Das Dashboard ist eine persoenliche Lernflaeche statt einer festen Startseite. Module koennen per Drag and Drop angeordnet werden; passende Modulgroessen erlauben kompakte Reihen, breite Uebersichten und gezielte Leerraeume.

Verfuegbare Module umfassen unter anderem:

- Stapel und Pruefungen als einzelne Module
- Lernlage, kleine Runde und Problemkarten
- Lernzeit, Lerntimer, Wochenplan und Meilensteine
- Tags, globale Suche und Einstellungen
- Einzelne frei waehlbare Karte mit Vorder-/Rueckseitenwechsel
- Archiv fuer archivierte oder ausgeblendete Stapel und Pruefungen
- Dekorative Spacer als Leerraum, Trennlinie oder Notizflaeche

Stapel und Pruefungen lassen sich archivieren, ohne ihre Karten oder Daten zu verlieren. Das Archiv kann ihre Sichtbarkeit auf dem Dashboard wiederherstellen.

## Import, Export und Anbindungen

### Daten austauschen

- **Anki-kompatibler Textimport/-export:** TSV, CSV oder Semikolon-getrennte Dateien mit Vorderseite, Rueckseite und optionalen Tags.
- **JSON-Backup:** Exportiert Stapel, Karten, Lernzustaende, Reviews, Einstellungen, Pruefungen und Vorlagen. Beim Wiederherstellen stehen Konfliktstrategien zur Verfuegung.
- **Obsidian:** Markdown-Dateien aus einem lokalen Vault anhand eines konfigurierbaren Tags importieren.

### Kartenquellen

In den Einstellungen stehen optionale Importer bereit:

| Quelle | Import |
| --- | --- |
| Zotero | Titel und Zusammenfassung aus der lokal laufenden Zotero-API |
| Notion | Zwei frei waehlbare Eigenschaften einer Notion-Datenquelle |
| Moodle | Begriffe und Definitionen aus einem Moodle-Glossar |

Zugangstokens werden nur fuer den jeweiligen Import verwendet und nicht gespeichert.

### URL-Schema

Andere Anwendungen oder Automationen koennen die Desktop-App ueber `stapelweise://` ansprechen:

```text
stapelweise://deck/new?name=Biologie
stapelweise://deck/open?deck=Biologie
stapelweise://card/new?deck=Biologie&front=Frage&back=Antwort
```

Parameter muessen URL-kodiert sein. Das Schema ist fuer die installierte Desktop-App registriert.

## Anpassung

- Theme: automatisch, hell oder dunkel
- Farbwelten: abgestimmte Akzentfarben fuer beide Modi
- Ueberschriften: Pixel- und normale Schriften zur Auswahl
- Karten: Serif oder Sans-Serif sowie drei Schriftgroessen
- Animationen: Kartenwechsel, Steuerung und Bewertungsleiste getrennt steuerbar
- Sprache der Oberflaeche: Deutsch, Englisch, Spanisch, Franzoesisch, Portugiesisch

## Architektur

```text
Svelte 5 + SvelteKit + Tailwind
            |
        Tauri IPC
            |
Rust Commands + SM-2 + Integrationen
            |
    SQLite (lokale Datenbank)
```

Das Frontend nutzt Svelte 5 Runes und ruft fachliche Rust-Commands ueber Tauri auf. Das Rust-Backend kapselt Persistenz, Backups, Importer und den SM-2-Algorithmus. Der Algorithmus ist als reine Funktion testbar und von SQLite getrennt.

## Technologie

| Bereich | Technologie |
| --- | --- |
| Desktop-App | Tauri 2 |
| Frontend | Svelte 5, SvelteKit, TypeScript |
| Styling | Tailwind CSS |
| Backend | Rust |
| Daten | SQLite mit rusqlite |
| Inhaltsrendering | marked und KaTeX |

## Entwicklung

### Voraussetzungen

- Node.js 20 oder neuer
- Rust stable mit Cargo
- Windows: Visual Studio Build Tools mit C++-Workload
- macOS: Xcode Command Line Tools
- Linux: eine WebKitGTK-Entwicklungsumgebung, zum Beispiel `libwebkit2gtk-4.1-dev` und `libgtk-3-dev`

### Setup

```bash
git clone https://github.com/cr4bbz/stapelweise.git
cd stapelweise
npm install
```

### Desktop-App starten

```bash
npm run tauri dev
```

Der Befehl startet die Tauri-App mit Frontend-Hot-Reload. Fuer einen Produktionsbuild:

```bash
npm run tauri build
```

### Qualitaetssicherung

```bash
# Svelte- und TypeScript-Pruefung
npm run check

# Rust-Tests
cd src-tauri
cargo test
```

## Mitwirken

Issues und Pull Requests sind willkommen. Bitte halte Aenderungen fokussiert, fuehre vor einem Pull Request mindestens `npm run check` und `cargo test` aus und beschreibe sichtbare Verhaltensaenderungen kurz in der PR.

## Lizenz

MIT. Details stehen in [LICENSE](LICENSE).
