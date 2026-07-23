<!-- Quick-capture module: add a basic card to a deck without leaving the
     dashboard. Refreshes aggregate stats on success. -->
<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { t } from "$lib/i18n";

  let selectedDeckId = $state("");
  let quickFront = $state("");
  let quickBack = $state("");
  let quickSaving = $state(false);
  let quickMessage = $state("");

  $effect(() => {
    if (!selectedDeckId && deckStore.decks[0]) selectedDeckId = deckStore.decks[0].id;
  });

  async function createQuickCard() {
    if (!selectedDeckId || !quickFront.trim() || !quickBack.trim()) return;
    quickSaving = true;
    quickMessage = "";
    try {
      await api.createCard(selectedDeckId, quickFront.trim(), quickBack.trim());
      quickFront = "";
      quickBack = "";
      quickMessage = t("Karte angelegt");
      dashboardStore.invalidate();
    } catch {
      quickMessage = t("Karte konnte nicht angelegt werden");
    } finally {
      quickSaving = false;
    }
  }
</script>

<form class="surface-panel h-full p-5" onsubmit={(event) => { event.preventDefault(); void createQuickCard(); }}>
  <div class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
    <p class="section-kicker mb-2">{t("Schnellerfassung")}</p>
    <select bind:value={selectedDeckId} class="module-accent-input rounded-md px-3 py-2 text-sm outline-none">
      {#each deckStore.decks as deck}<option value={deck.id}>{deck.name}</option>{/each}
    </select>
  </div>
  <div class="mt-4 grid gap-3 sm:grid-cols-2">
    <input bind:value={quickFront} aria-label={t("Vorderseite")} placeholder={t("Vorderseite")} class="module-accent-input rounded-md px-3 py-2.5 text-sm outline-none" />
    <input bind:value={quickBack} aria-label={t("Rückseite")} placeholder={t("Rückseite")} class="module-accent-input rounded-md px-3 py-2.5 text-sm outline-none" />
  </div>
  <div class="mt-3 flex items-center justify-between gap-3">
    <span class="text-xs text-secondary" aria-live="polite">{quickMessage}</span>
    <button type="submit" disabled={quickSaving || !selectedDeckId || !quickFront.trim() || !quickBack.trim()} class="primary-action px-4 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-40">{quickSaving ? t("Speichert...") : t("Karte anlegen")}</button>
  </div>
</form>
