# Stapelweise – Vision

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

## 2. Offline First und dauerhaft ohne eigene Cloud

Die Anwendung muss vollständig ohne Internet funktionieren.

Internet ist optional und darf nur für ausdrücklich vom Nutzer gewählte externe Integrationen benötigt werden.

Stapelweise selbst erhält dauerhaft keine eigene Cloud-Funktion, keine verpflichtende Serverkomponente und keinen Account-Zwang. Lokale Nutzung bleibt das vollständige Produkt, nicht ein eingeschränkter Offline-Modus.

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


# Bewusste Produktgrenzen

Folgende Funktionen werden dauerhaft nicht Teil des Kernprodukts:

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
