<script lang="ts">
  import * as api from "$lib/api";
  import type { ImportInspection } from "$lib/types";

  type ConflictStrategy = "overwrite" | "skip" | "merge";

  let fileInput: HTMLInputElement;
  let jsonData = $state("");
  let fileName = $state("");
  let inspection = $state<ImportInspection | null>(null);
  let strategy = $state<ConflictStrategy>("merge");
  let busy = $state(false);
  let status = $state("");
  let error = $state("");

  function backupFileName() {
    return `stapelweise-backup-${new Date().toISOString().slice(0, 10)}.json`;
  }

  async function exportBackup(download: boolean) {
    busy = true;
    status = "";
    error = "";
    try {
      const data = await api.exportBackupData();
      if (download) {
        const blob = new Blob([data], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement("a");
        anchor.href = url;
        anchor.download = backupFileName();
        anchor.click();
        URL.revokeObjectURL(url);
        status = "Backup-Datei erstellt.";
      } else {
        await navigator.clipboard.writeText(data);
        status = "Backup-JSON in die Zwischenablage kopiert.";
      }
    } catch (cause) {
      error = `Export fehlgeschlagen: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function inspectFile(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;
    busy = true;
    status = "";
    error = "";
    inspection = null;
    try {
      jsonData = await file.text();
      fileName = file.name;
      inspection = await api.inspectBackupData(jsonData);
      status = "Backup geprüft. Es wurde noch nichts verändert.";
    } catch (cause) {
      jsonData = "";
      fileName = file.name;
      error = `Backup konnte nicht geprüft werden: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function restore() {
    if (!inspection || !jsonData) return;
    busy = true;
    status = "";
    error = "";
    try {
      await api.restoreBackupData(jsonData, strategy);
      status = "Backup wiederhergestellt. Öffne die Bibliothek neu, um alle Änderungen zu sehen.";
      inspection = null;
      jsonData = "";
      fileName = "";
      if (fileInput) fileInput.value = "";
    } catch (cause) {
      error = `Wiederherstellung fehlgeschlagen: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }
</script>

<div class="space-y-4 rounded-card border border-secondary/20 p-4">
  <div>
    <span class="text-sm font-medium text-primary dark:text-primary-dark">JSON-Backup</span>
    <p class="mt-1 text-xs text-secondary">
      Exportiert deine lokalen Stapel, Karten, Lernstände, Reviews, Einstellungen und Prüfungsdaten.
    </p>
  </div>

  <div class="flex flex-wrap gap-2">
    <button
      type="button"
      disabled={busy}
      onclick={() => exportBackup(true)}
      class="rounded-button bg-accent-correct px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50"
    >
      Backup herunterladen
    </button>
    <button
      type="button"
      disabled={busy}
      onclick={() => exportBackup(false)}
      class="rounded-button bg-white/40 px-4 py-1.5 text-sm font-medium text-primary dark:bg-white/10 dark:text-primary-dark disabled:opacity-50"
    >
      JSON kopieren
    </button>
  </div>

  <div class="border-t border-secondary/20 pt-4">
    <input
      bind:this={fileInput}
      type="file"
      accept="application/json,.json"
      onchange={inspectFile}
      class="block w-full text-xs text-secondary file:mr-3 file:rounded-button file:border-0 file:bg-white/40 file:px-3 file:py-1.5 file:text-sm file:font-medium dark:file:bg-white/10"
    />
  </div>

  {#if inspection}
    <div class="space-y-3 rounded-card bg-black/5 p-3 text-xs dark:bg-black/20">
      <div class="font-medium text-primary dark:text-primary-dark">Vorschau: {fileName}</div>
      <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-secondary sm:grid-cols-3">
        <span>{inspection.deck_count} Stapel</span>
        <span>{inspection.card_count} Karten</span>
        <span>{inspection.review_count} Reviews</span>
        <span>{inspection.exam_count} Prüfungen</span>
        <span>{inspection.template_count} Vorlagen</span>
        <span>{inspection.existing_card_conflicts} Kartenkonflikte</span>
      </div>
      {#if inspection.existing_deck_conflicts.length > 0}
        <p class="text-secondary">
          Bestehende Stapel-IDs: {inspection.existing_deck_conflicts.join(", ")}
        </p>
      {/if}
      {#each inspection.warnings as warning}
        <p class="text-amber-700 dark:text-amber-300">{warning}</p>
      {/each}

      <label class="block">
        <span class="mb-1 block font-medium text-primary dark:text-primary-dark">Konfliktstrategie</span>
        <select
          bind:value={strategy}
          class="w-full rounded-md border border-secondary/30 bg-transparent px-3 py-2 text-sm text-primary dark:text-primary-dark"
        >
          <option value="merge">Zusammenführen – Backup-Inhalte, lokale Lernstände & Einstellungen behalten</option>
          <option value="skip">Überspringen – vorhandene Datensätze nicht verändern</option>
          <option value="overwrite">Überschreiben – Backup-Daten gewinnen</option>
        </select>
      </label>

      <button
        type="button"
        disabled={busy}
        onclick={restore}
        class="rounded-button bg-accent-correct px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50"
      >
        Wiederherstellen
      </button>
    </div>
  {/if}

  {#if status}<p class="text-xs text-accent-correct">{status}</p>{/if}
  {#if error}<p class="text-xs text-red-600 dark:text-red-300">{error}</p>{/if}
</div>
