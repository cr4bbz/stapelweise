<p align="center">
  <img src="" width="0" alt="">
  <h1 align="center">Stapelweise</h1>
  <p align="center">
    Moderne Karteikarten-App mit Spaced Repetition — lokal, offline, kostenlos.<br/>
    Die Open-Source-Alternative zu Anki.
  </p>
</p>

<p align="center">
  <a href="https://github.com/cr4bbz/stapelweise/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://github.com/cr4bbz/stapelweise/blob/main/PRODUCT_MANIFEST.md"><img src="https://img.shields.io/badge/version-v1.0.0-green.svg" alt="v1.0 Release"></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=white" alt="Tauri 2"></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white" alt="Svelte 5"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.80+-DEA584?logo=rust&logoColor=white" alt="Rust"></a>
</p>

---

## Inhaltsverzeichnis

- [Features](#features)
- [Architektur](#architektur)
- [Tech-Stack](#tech-stack)
- [Entwicklung](#entwicklung)
  - [Voraussetzungen](#voraussetzungen)
  - [Setup](#setup)
  - [Dev-Server](#dev-server)
  - [Produktiv-Build](#produktiv-build)
- [Mitwirken](#mitwirken)
- [Lizenz](#lizenz)

## Features

### Lernen & Wiederholen

- **SM-2-Algorithmus** — bewährter Spaced-Repetition-Algorithmus mit konfigurierbaren Parametern (initialer Ease-Faktor, Bestehens-Schwelle)
- **Multi-Deck-Lernen** — mehrere Stapel gleichzeitig in einer Sitzung lernen, Karten gemischt
- **Session-Limit** — konfigurierbare maximale Kartenanzahl pro Lernsitzung
- **Rückgängig** — letzte Bewertung per `Ctrl+Z` rückgängig machen
- **Durchblättern** — Karten vor dem Lernen durchblättern und mit `Space` wenden

### Inhalte

- **LaTeX-Formeln** — `$...$` für inline, `$$...$$` für abgesetzte Formeln, gerendert mit KaTeX
- **Markdown** — **fett**, *kursiv*, Listen, Code-Blöcke und mehr
- **Live-Vorschau** — LaTeX- und Markdown-Rendering direkt während der Eingabe
- **XSS-Schutz** — rohes HTML wird entfernt, gefährliche URLs blockiert

### Organisation

- **Suche** — globale Volltextsuche über alle Karten und Stapel hinweg
- **Import/Export** — Stapel als CSV exportieren und importieren (in Arbeit)
- **Beispieldaten** — drei Beispiel-Stapel zum Ausprobieren auf Knopfdruck

### Statistiken

- **Dashboard** — fällige Karten, heutige Reviews, Tagesserie (Streak)
- **Deck-Statistiken** — Karten nach Status (Neu, Lernend, Wiederholend, Gelernt), durchschnittlicher Ease-Faktor und Intervall

### Anpassung

- **Theme** — Hell, Dunkel oder automatisch nach System
- **Schriftart** — Serif oder Sans-Serif für Karteninhalte
- **Schriftgröße** — Klein, Mittel oder Groß für optimale Lesbarkeit
- **Fenster-Persistenz** — Fenstergröße und -position bleiben über Neustarts erhalten
- **Tastenkürzel** — alle Shortcuts via `?`-Taste einsehbar

### Plattform

- **Lokal & Offline** — alle Daten in lokalem SQLite, kein Account, kein Internet nötig
- **Cross-Platform** — Windows, macOS, Linux aus einer Codebasis

## Architektur

```
┌─────────────────────────────────────────┐
│               Frontend                  │
│    Svelte 5 + SvelteKit + Tailwind     │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │  Routes  │ │  Stores  │ │   Lib   │ │
│  │  +page   │ │  decks   │ │  api.ts │ │
│  │  Layout  │ │  study   │ │ math.ts │ │
│  │          │ │settings  │ │markdown │ │
│  └──────────┘ └──────────┘ └─────────┘ │
├─────────────────────────────────────────┤
│           Tauri IPC (invoke)            │
├─────────────────────────────────────────┤
│               Backend (Rust)            │
│                                         │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │ Commands │ │    DB    │ │   SRS   │ │
│  │  decks   │ │ models   │ │   SM-2  │ │
│  │  cards   │ │repository│ │         │ │
│  │ settings │ │migrations│ │         │ │
│  │  stats   │ │          │ │         │ │
│  └──────────┘ └──────────┘ └─────────┘ │
├─────────────────────────────────────────┤
│           SQLite (rusqlite)             │
└─────────────────────────────────────────┘
```

Die App folgt einem **Command-Muster**: Das Svelte-Frontend ruft Rust-Commands via Tauri IPC auf. Jeder Command wickelt eine fachliche Operation ab (z.B. `create_card`, `submit_review`) und kommuniziert über das `Repository` mit SQLite. Der SM-2-Algorithmus ist als reine Funktion implementiert und hat keine Datenbank-Abhängigkeit.

## Tech-Stack

| Bereich | Technologie | Version |
|---------|------------|---------|
| Desktop-Framework | [Tauri](https://tauri.app) | 2.x |
| Frontend | [Svelte](https://svelte.dev) (Runes) + [SvelteKit](https://kit.svelte.dev) | 5.x |
| Styling | [Tailwind CSS](https://tailwindcss.com) | 3.x |
| Backend | [Rust](https://www.rust-lang.org) | 1.80+ |
| Datenbank | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) | 0.31 (bundled) |
| Math-Rendering | [KaTeX](https://katex.org) | 0.17 |
| Markdown | [marked](https://marked.js.org) | 18.x |
| Datumslogik | [chrono](https://github.com/chronotope/chrono) | 0.4 |

## Entwicklung

### Voraussetzungen

- [Node.js](https://nodejs.org) >= 20
- [Rust](https://rustup.rs) >= 1.80
- Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) mit C++-Workload
- macOS: Xcode Command Line Tools (`xcode-select --install`)
- Linux: `build-essential`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`

### Setup

```bash
git clone https://github.com/cr4bbz/stapelweise.git
cd stapelweise
npm install
```

### Dev-Server

```bash
npm run tauri dev
```

Startet die App mit Hot-Reload für das Frontend. Der Rust-Backend wird bei Änderungen neu kompiliert.

### Produktiv-Build

```bash
npm run tauri build
```

Erzeugt die native App im `src-tauri/target/release/`-Verzeichnis.

### Tests

```bash
# Rust-Tests
cd src-tauri && cargo test

# Frontend-Type-Check
npm run check
```

## Mitwirken

Beiträge sind willkommen! Ob Bug-Reports, Feature-Vorschläge oder Pull Requests — schau dich um und mach mit.

1. **Fork** das Repository
2. **Branch** anlegen: `feature/coole-idee` oder `fix/bug-beschreibung`
3. **Änderungen** umsetzen
4. **Checks** ausführen: `npm run check && cd src-tauri && cargo test`
5. **Pull Request** erstellen mit Beschreibung der Änderung

Bei größeren Features bitte vorher ein Issue anlegen, damit wir den Ansatz besprechen können.

## Lizenz

MIT — siehe [LICENSE](LICENSE).
