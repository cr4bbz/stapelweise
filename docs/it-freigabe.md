# IT-Freigabe – Stapelweise

Dieses Dokument fasst die technischen Eigenschaften zusammen, die für eine lokale Installation oder institutionelle Freigabe relevant sind.

## Betriebsmodell

Stapelweise ist eine Desktop-Anwendung auf Basis von Tauri 2. Die Kernanwendung benötigt weder Benutzerkonto noch Stapelweise-Server noch Cloud-Synchronisation. Karten, Lernstände, Einstellungen und Prüfungsdaten werden lokal in SQLite gespeichert.

## Netzwerk

Die Kernfunktionen arbeiten offline. Netzwerkzugriffe entstehen nur durch vom Nutzer ausdrücklich gestartete optionale Integrationen, etwa Notion oder Moodle. Zotero wird über dessen lokale API angesprochen. Der Obsidian-Importer liest lokale Markdown-Dateien und schreibt nicht in den Vault zurück.

Stapelweise betreibt keine eigene Cloud-Funktion und überträgt die lokale Lerndatenbank nicht automatisch an einen Server.

## Lokale Daten

- SQLite-Datenbank im anwendungsspezifischen Datenverzeichnis des Betriebssystems
- JSON-Backup für vollständige lokale Datensicherung und Wiederherstellung
- optionale lokale Medienreferenzen
- keine Kontodaten für Stapelweise selbst

## Rechte

Für den normalen Betrieb sind Standard-Benutzerrechte vorgesehen. Zusätzliche Dateizugriffe erfolgen nur auf vom Nutzer ausgewählte lokale Pfade, beispielsweise einen Obsidian-Vault. Installation und Paketverwaltung können je nach Betriebssystem administrative Rechte erfordern.

## Externe Integrationen

Zugangstokens für optionale Drittanbieter werden nur für den jeweiligen Import verwendet und sind keine Voraussetzung für die Anwendung. Organisationen können diese Integrationen ungenutzt lassen und Stapelweise vollständig offline betreiben.

## MCP

Der optionale MCP-Modus stellt eine lokale Schnittstelle für kompatible Assistenten bereit. Er ist kein Cloud-Dienst von Stapelweise. Ob ein angebundener Assistent Daten an einen externen Dienst übermittelt, hängt ausschließlich von dessen eigener Konfiguration ab.

## Distribution und Integrität

Der Linux-Release-Workflow erzeugt `.deb`, `.rpm` und `.AppImage` und veröffentlicht eine `SHA256SUMS`-Datei. Die Prüfsumme ermöglicht die Kontrolle, ob ein heruntergeladenes Paket unverändert ist.

## Lizenz

Stapelweise steht unter der MIT-Lizenz. Der Quellcode und die Lizenzbedingungen befinden sich im öffentlichen Repository.

## Ansprechpartner für eine Freigabeprüfung

Für eine institutionelle Prüfung sollten mindestens Repository-Revision, Release-Tag, SHA256-Prüfsumme, eingesetztes Betriebssystem und die tatsächlich erlaubten optionalen Integrationen dokumentiert werden.
