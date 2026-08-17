# Stapelweise – Linux-Testrelease

Dieses Dokument begleitet den vorläufigen Linux-Testrelease `v0.1.0-rc.1`. Der Release ist ausdrücklich ein **Release Candidate für Installation, IT-Prüfung und Praxistests** und noch keine endgültige Produktfreigabe.

## Zweck

Der Release soll insbesondere klären, ob Stapelweise auf realen Linux-Arbeitsplätzen sauber installierbar ist, mit Standard-Benutzerrechten zuverlässig läuft und sich das dokumentierte lokale/offline Betriebsmodell in einer administrierten Umgebung bestätigt.

## Veröffentlichte Artefakte

Der GitHub-Release-Workflow erzeugt:

- Debian-Paket (`.deb`)
- RPM-Paket (`.rpm`)
- AppImage (`.AppImage`)
- `SHA256SUMS` mit Prüfsummen aller Pakete

Alle Artefakte stammen aus demselben Git-Commit des Release-Tags.

## Integrität prüfen

Im Verzeichnis mit den heruntergeladenen Dateien:

```bash
sha256sum -c SHA256SUMS
```

Für jede vorhandene Paketdatei sollte `OK` ausgegeben werden. Dieser Release verwendet Prüfsummen zur Integritätskontrolle; eine separate Paket- oder Code-Signatur ist für diesen Release Candidate noch nicht eingerichtet.

## Installation

### Debian / Ubuntu

```bash
sudo apt install ./<stapelweise-datei>.deb
```

### Fedora / RHEL-kompatible Systeme

```bash
sudo dnf install ./<stapelweise-datei>.rpm
```

### AppImage

```bash
chmod +x ./<stapelweise-datei>.AppImage
./<stapelweise-datei>.AppImage
```

Je nach Distribution kann für AppImages FUSE-Kompatibilität benötigt werden. Falls das AppImage nicht startet, bitte Distribution, Version und Fehlermeldung dokumentieren und zusätzlich das native `.deb`- beziehungsweise `.rpm`-Paket testen, sofern passend.

## Minimaler Admin-Test

Bitte nach Möglichkeit mit einem normalen Benutzerkonto testen.

1. Anwendung installieren beziehungsweise AppImage starten.
2. Prüfen, dass kein Stapelweise-Konto oder Login verlangt wird.
3. Einen Stapel und mehrere Karten anlegen.
4. Anwendung schließen und erneut öffnen; lokale Daten müssen erhalten bleiben.
5. Eine kurze Lernrunde durchführen und erneut starten; der Lernstand muss erhalten bleiben.
6. Ein JSON-Backup exportieren.
7. Für einen Wiederherstellungstest ausschließlich Testdaten verwenden und die Konfliktstrategie bewusst auswählen.
8. Ohne optionale Integrationen prüfen, ob die Kernfunktionen offline nutzbar bleiben.
9. Optional den beobachteten Netzwerkverkehr dokumentieren. Netzwerkzugriffe sollen bei Kernfunktionen nicht erforderlich sein; externe Integrationen werden nur durch Nutzeraktion angestoßen.
10. Deinstallation beziehungsweise Entfernen des AppImages testen und dokumentieren, ob lokale Anwendungsdaten bewusst erhalten oder separat gelöscht werden sollen.

## Besonders relevante Prüfpunkte für IT

- verwendete Distribution und Version
- Desktop-Umgebung
- verwendetes Artefakt (`.deb`, `.rpm` oder `.AppImage`)
- SHA256-Prüfung erfolgreich / nicht erfolgreich
- Installation mit oder ohne administrative Rechte
- Start mit Standard-Benutzerrechten
- Speicherort und Rechte der lokalen Anwendungsdaten
- Verhalten ohne Netzwerkverbindung
- beobachtete ausgehende Verbindungen bei Nutzung der Kernfunktionen
- Verhalten von Backup und Wiederherstellung
- Fehlermeldungen beim Start oder bei Paketabhängigkeiten

## Daten- und Netzwerkmodell

Stapelweise speichert Karten, Lernstände, Einstellungen und Prüfungsdaten lokal in SQLite. Die Kernanwendung benötigt weder Stapelweise-Konto noch Stapelweise-Server noch Cloud-Synchronisation. Optionale Drittanbieter-Integrationen sind nicht erforderlich und werden nur auf Nutzeraktion verwendet.

Für die institutionelle Einordnung siehe zusätzlich [`it-freigabe.md`](./it-freigabe.md).

## Bekannte Grenze dieses Release Candidates

Die automatisierten Frontend- und Rust-Tests sind Bestandteil der CI. Der Release Candidate dient zusätzlich gerade dazu, die **gebauten Linux-Pakete, Installation, Desktop-Laufzeit und reale UI-Nutzung** auf einem unabhängigen System zu prüfen. Ein erfolgreicher CI-Lauf ersetzt diese Praxistests nicht.

## Fehlerbericht

Ein hilfreicher Fehlerbericht enthält mindestens:

- Release-Tag und SHA256-Prüfstatus
- Distribution, Version und Architektur
- verwendetes Paketformat
- konkrete Schritte bis zum Fehler
- erwartetes und tatsächliches Verhalten
- vollständige Fehlermeldung, falls vorhanden
- Angabe, ob der Fehler auch nach Neustart reproduzierbar ist

Keine privaten Lerninhalte, Zugangstokens oder vollständigen persönlichen Datenbanken an einen Fehlerbericht anhängen.
