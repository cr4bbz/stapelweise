# Stapelweise v1.0 – Vision & Implementierungsroadmap

## Projektphilosophie

**Stapelweise ist keine Alternative zu Anki oder Quizlet.**

Stapelweise verfolgt ein anderes Ziel:

> **Verstehen statt bloßes Erinnern.**

Jede Funktion muss sich an folgender Frage messen lassen:

> **Hilft diese Funktion dem Nutzer dabei, Zusammenhänge besser zu verstehen?**

Falls die Antwort "Nein" lautet, gehört sie nicht in Stapelweise.

---

# Kernprinzipien

## 1. Der Nutzer besitzt seine Daten.

* Keine proprietären Dateiformate.
* Lokale Speicherung als Standard.
* Offene Export- und Importformate.
* Keine künstliche Bindung an die Plattform.

Ziel:

> Der Nutzer soll Stapelweise jederzeit verlassen können, ohne Wissen zu verlieren.

---

## 2. Offline First

Die Anwendung muss vollständig ohne Internet funktionieren.

Internet ist optional.

Nicht Voraussetzung.

---

## 3. Open Source

Der Quellcode bleibt nachvollziehbar.

Die Community soll Erweiterungen entwickeln können.

---

## 4. Minimalismus

Keine Funktion wird implementiert, weil andere Lernplattformen sie besitzen.

Jede Funktion benötigt eine klare didaktische Begründung.

---

# Zielgruppe

Nicht:

* Schüler, die möglichst schnell Prüfungen bestehen möchten.

Sondern:

Menschen,

* die verstehen möchten,
* die Zusammenhänge erkennen wollen,
* die langfristig lernen,
* die Freude an Wissen besitzen.

---

# Definition von Version 1.0

Version 1.0 ist erreicht,

wenn ein neuer Nutzer Stapelweise herunterladen kann und ohne Hilfe effizient lernen kann.

Nicht wenn sämtliche Ideen implementiert wurden.

---

# Roadmap

---

## Phase 1 – Fundament

### Karten

Unterstützte Kartentypen

* Basic
* Cloze
* Multiple Choice
* Reihenfolge
* Freitext

---

### Markdown

Vollständige Markdown-Unterstützung

* Überschriften
* Tabellen
* Listen
* Codeblöcke
* Zitate

---

### LaTeX

Vollständiges Rendering.

Mathematik muss zuverlässig funktionieren.

---

### Bilder

* Bilder einfügen
* Bilder zuschneiden
* Bildausschnitte verwenden
* Annotationen

---

### Lokale Datenbank

SQLite

Ziel:

Eine einzige robuste Datenbankdatei.

---

### Suche

Sehr leistungsfähig.

Unterstützung für

* Fuzzy Search
* Tags
* Markdown
* LaTeX
* Regex

Die Suche soll sich wie eine Wissenssuche anfühlen.

---

### Tags

Decks bleiben möglich.

Tags werden jedoch zum primären Organisationssystem.

Ein Inhalt kann beliebig viele Tags besitzen.

---

## Phase 2 – Lernen

### Spaced Repetition

Eigenes oder bestehendes Scheduling.

Konfigurierbar.

---

### Testmodus

Nicht nur einzelne Karten.

Sondern vollständige Tests.

Parameter:

* Zeitlimit
* Fragenanzahl
* Kartentypen mischen
* Punkte
* Ergebnisübersicht

Ziel:

Simulation echter Prüfungen.

---

### Lernstatistiken

Nicht Gamification.

Sondern sinnvolle Lernanalyse.

Beispiele:

* Schwierige Themen
* Sichere Themen
* Wiederholungsbedarf
* Langfristige Entwicklung

Keine XP.

Keine künstlichen Streaks.

---

## Phase 3 – Integration

### Obsidian-Konnektor

Direkte Nutzung vorhandener Markdown-Dateien im Obsidian-Vault.

Keine Kopien. Keine Synchronisationsprobleme.

**Funktionsweise:**
* Stapelweise überwacht einen ausgewählten Ordner (Vault).
* Notizen mit bestimmten Tags (z. B. `#flashcard`) oder spezieller Formatierung (z.B. Fragen-Antwort-Blöcke) werden automatisch als Karten erkannt.
* Der Lernfortschritt wird separat in der Stapelweise-Datenbank (SQLite) gespeichert, sodass die Obsidian-Dateien nicht mit Metadaten verschmutzt werden.
* Der ursprüngliche Kontext der Notiz kann beim Lernen mit einem Klick eingeblendet werden.

---

### Plugin-System

Öffentliche API.

Dritte sollen Erweiterungen entwickeln können.

---

### Import

Unterstützung für

* Markdown
* CSV
* JSON
* Anki

---

### Export

Unterstützung für

* Markdown
* CSV
* JSON
* PDF
* Anki

---

## Phase 4 – Erweiterte Funktionen

### PDF-Unterstützung

Text markieren

↓

Direkt Karte erstellen.

---

### MCP-Unterstützung (Model Context Protocol)

Anbindung **beliebiger** LLMs über das offene Model Context Protocol. Nicht auf Claude beschränkt, sondern agnostisch für alle modernen KI-Modelle (OpenAI, lokale Modelle via Ollama etc.).

Stapelweise kann in zwei Rollen agieren:

**1. Stapelweise als MCP-Server:**
Externe KI-Assistenten (z. B. Claude Desktop) können auf das Wissen in Stapelweise zugreifen.
* Nutzer können mit ihrer KI über ihren Lernfortschritt chatten.
* Die KI kann im Auftrag des Nutzers neue Karten anlegen (z. B. "Erstelle mir Karteikarten aus meinem letzten PDF und speichere sie in Stapelweise").

**2. Stapelweise als MCP-Client:**
Einbindung lokaler oder Cloud-basierter LLMs direkt in die Lern-App.
* "Grill me"-Modus: Die KI fragt das Wissen dialogisch ab.
* Automatische Erstellung von "Warum?"-Erklärungen, wenn der Nutzer nicht weiterweiß.

**Grenzen der KI:**
LLMs dienen ausschließlich als Werkzeug.
Sie ersetzen niemals den eigenen Lernprozess.
Keine verpflichtende Cloud-Abhängigkeit – lokale Modelle haben Priorität.

---

# Bewusst NICHT Bestandteil von Version 1

Folgende Funktionen gehören ausdrücklich nicht zu v1:

* Benutzerkonten
* Cloud-Synchronisation
* Soziale Netzwerke
* Werbung
* Premium-Features
* Bezahlte Lerninhalte
* Marketplace
* Gamification um ihrer selbst willen
* KI als Kernfunktion

Stapelweise muss auch ohne LLMs eine hervorragende Lernplattform sein.

---

# Besonderes Feature: "Warum?"

Jede Karte besitzt optional einen Bereich:

## Warum?

Der Nutzer beantwortet für sich selbst:

* Warum ist diese Aussage wahr?
* Wie hängt sie mit anderem Wissen zusammen?
* Welche Begründung steckt dahinter?
* Welche Annahmen werden gemacht?

Diese Antworten werden nicht bewertet.

Sie dienen ausschließlich der aktiven Wissenskonstruktion.

Dieses Feature repräsentiert die Philosophie von Stapelweise.

---

# Softwarearchitektur

Empfohlene Grundprinzipien

* Modular
* Offline First
* SQLite
* Klare Trennung von UI und Logik
* Erweiterbar
* Testbar

---

# Langfristige Vision

Stapelweise ist kein einzelnes Produkt.

Es ist der erste Baustein eines offenen Lernökosystems.

Andere Projekte (z. B. Logos Table) bleiben eigenständige Anwendungen, können jedoch über offene Schnittstellen tief integriert werden.

Keine Fusion.

Lose Kopplung.

Klare Verantwortlichkeiten.

---

# Leitbild

> Stapelweise ist eine Open-Source-Lernplattform für Menschen, die verstehen statt nur wiederholen wollen.

Nicht Aufmerksamkeit maximieren.

Nicht Bildschirmzeit maximieren.

Nicht Profit maximieren.

Sondern Verständnis fördern.

---

# Entscheidungsregel für zukünftige Features

Vor jeder neuen Funktion wird geprüft:

1. Fördert sie Verständnis?
2. Macht sie das Lernen klarer?
3. Passt zur Philosophie von Offline First?
4. Bleiben die Daten vollständig im Besitz des Nutzers?
5. Erhöht sie die Komplexität ohne entsprechenden didaktischen Mehrwert?

Falls eine dieser Fragen negativ beantwortet wird, sollte die Funktion nicht implementiert werden.
