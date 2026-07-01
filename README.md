# Stapelweise

Moderne Karteikarten-Lernplattform mit Spaced Repetition. Kostenlos lokal, günstig in der Cloud. Die Open-Source-Alternative zu Anki.

## Status

🚧 In früher Entwicklung — MVP in Arbeit.

## Features

- **SM-2 Spaced Repetition** — bewährter Algorithmus mit konfigurierbaren Parametern
- **LaTeX-Support** — mathematische Formeln mit `$...$` (inline) und `$$...$$` (display)
- **Live-Vorschau** — LaTeX-Rendering direkt während der Kartenerstellung
- **Statistiken** — Dashboard mit fälligen Karten, heutigen Reviews und Streak; Deck-Statistiken mit Kategorien (Neu/Lernend/Wiederholend/Gelernt)
- **Anpassbare UI** — Theme (Hell/Dunkel/Auto), Schriftart und Schriftgröße für Karten
- **Session-Limit** — konfigurierbare Anzahl Karten pro Lernsitzung
- **Fenster-Persistenz** — Fenstergröße und -position bleiben erhalten
- **Lokal & Offline** — alle Daten in lokalem SQLite, kein Account nötig
- **Cross-Platform** — Windows, macOS, Linux

## Tech-Stack

| Bereich | Technologie |
|---------|------------|
| Desktop-Framework | [Tauri 2](https://tauri.app) |
| Frontend | [Svelte 5](https://svelte.dev) (Runes) + [SvelteKit](https://kit.svelte.dev) |
| Styling | [Tailwind CSS 3](https://tailwindcss.com) |
| Backend | Rust |
| Datenbank | SQLite (via `rusqlite`) |
| Math-Rendering | [KaTeX](https://katex.org) |

## Entwicklung

### Voraussetzungen

- [Node.js](https://nodejs.org) ≥ 20
- [Rust](https://rustup.rs) ≥ 1.80
- Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) mit C++-Workload

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

### Produktiv-Build

```bash
npm run tauri build
```

## Lizenz

MIT
