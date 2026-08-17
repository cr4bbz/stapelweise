# Stapelweise – Roadmap

Diese Datei beschreibt geplante Arbeit. Sie ist bewusst von `VISION.md` (Produktabsicht), `PRODUCT_MANIFEST.md` (gegenwärtiger Funktionsstand) und `README.md` (gegenwärtige Nutzung) getrennt.

## Leitplanken

- Stapelweise bleibt vollständig lokal nutzbar und erhält dauerhaft keine eigene Cloud-Funktion oder Kontopflicht.
- Neue Funktionen müssen Verständnis fördern, Datenhoheit erhalten und die lokale Architektur nicht unnötig verkomplizieren.
- Zukunftspläne werden erst nach Umsetzung in `PRODUCT_MANIFEST.md` und `README.md` übernommen.

## Etappe 3 – Wissensstruktur

- Tags als primäres, deckübergreifendes Organisationsmittel weiter ausbauen.
- Beziehungen zwischen Karten und Themen sichtbarer machen, ohne eine zweite proprietäre Wissensdatenbank einzuführen.
- Bestehende Such- und Filterpfade auf konsistente Tag-Nutzung prüfen.

## Etappe 4 – Suche

- SQLite FTS5 evaluieren und die derzeitige `LIKE`-Suche ablösen, sofern Migration, Ranking und Offline-Verhalten robust bleiben.
- Suchsyntax für Tags, Inhalte und optionale strukturierte Filter definieren.

## Etappe 5 – „Warum?“-Ebene

- Das vorhandene Begründungsfeld stärker als Werkzeug für aktive Wissenskonstruktion nutzbar machen.
- Auswertungen sollen Verständnislücken sichtbar machen, ohne XP- oder Aufmerksamkeitsmechaniken einzuführen.

## Etappe 6 – Offene Integrationen

- MCP als lokale, optionale Schnittstelle weiter härten und dokumentieren.
- Externe Integrationen bleiben opt-in; Tokens und fremde Dienste dürfen keine Voraussetzung für Kernfunktionen werden.
- Obsidian bleibt eine lokale Dateiquelle. Eine Schreib-Synchronisation ist nicht Teil des aktuellen Produkts.

## Offene Produktfrage

- Wie weit soll die Verschiebung von Decks zu Tags als primärem Organisationsmodell gehen, ohne die einfache mentale Struktur klassischer Stapel zu verlieren?
