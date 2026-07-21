<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { t } from "$lib/i18n";

  type Service = "zotero" | "notion" | "moodle";
  let activeService = $state<Service | null>(null);
  let loading = $state<Service | null>(null);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);

  let zoteroDeckName = $state("Zotero");
  let zoteroCollectionKey = $state("");

  let notionDeckName = $state("Notion");
  let notionToken = $state("");
  let notionDataSourceId = $state("");
  let notionFrontProperty = $state("Vorderseite");
  let notionBackProperty = $state("Rückseite");

  let moodleDeckName = $state("Moodle");
  let moodleBaseUrl = $state("");
  let moodleToken = $state("");
  let moodleGlossaryId = $state(0);

  const serviceHelp: Record<Service, string> = {
    zotero: "Importiert Titel und Zusammenfassung aus deiner lokal geöffneten Zotero-Bibliothek. Der Collection-Key ist optional.",
    notion: "Teile die Datenquelle zuvor mit deiner Notion-Integration. Die Datenquellen-ID steht in ihrer Notion-URL.",
    moodle: "Nutze die Basis-URL deines Moodle und einen Webservice-Token mit Zugriff auf das Glossar.",
  };

  function resultLabel(service: string, result: { imported: number; skipped: number }) {
    return `${service}: ${result.imported} ${t("Karten importiert")}${result.skipped ? `, ${result.skipped} ${t("übersprungen")}` : ""}.`;
  }

  async function afterImport(service: string, request: () => Promise<{ imported: number; skipped: number }>) {
    loading = service as Service;
    error = null;
    message = null;
    try {
      const result = await request();
      await deckStore.load();
      message = resultLabel(service, result);
    } catch (reason: any) {
      error = t("Import fehlgeschlagen.");
    } finally {
      loading = null;
    }
  }
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between gap-4">
    <span class="text-sm font-medium text-primary dark:text-primary-dark">{t("Kartenquellen")}</span>
    {#if message}<span class="text-xs font-medium text-accent-correct">{message}</span>{/if}
  </div>
  <p class="text-xs text-secondary">{t("Zugangstokens gelten nur für den laufenden Import und werden nicht gespeichert.")}</p>
  {#if error}<p class="text-xs text-accent-incorrect">{error}</p>{/if}

  <div class="flex flex-wrap gap-2">
    <button class="{activeService === 'zotero' ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs" aria-pressed={activeService === "zotero"} onclick={() => (activeService = activeService === "zotero" ? null : "zotero")}>Zotero</button>
    <button class="{activeService === 'notion' ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs" aria-pressed={activeService === "notion"} onclick={() => (activeService = activeService === "notion" ? null : "notion")}>Notion</button>
    <button class="{activeService === 'moodle' ? 'primary-action' : 'secondary-action'} px-3 py-1.5 text-xs" aria-pressed={activeService === "moodle"} onclick={() => (activeService = activeService === "moodle" ? null : "moodle")}>Moodle</button>
  </div>
  {#if activeService}<p class="text-xs text-secondary">{t(serviceHelp[activeService])}</p>{/if}

  {#if activeService === "zotero"}
    <form class="surface-panel grid gap-3 p-4 sm:grid-cols-2" onsubmit={(event) => { event.preventDefault(); afterImport("Zotero", () => api.importZoteroLocal(zoteroDeckName, zoteroCollectionKey || null)); }}>
      <label class="text-xs text-secondary">{t("Zielstapel")}<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={zoteroDeckName} required /></label>
      <label class="text-xs text-secondary">Collection-Key <input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={zoteroCollectionKey} placeholder="optional" /></label>
      <button class="primary-action w-fit px-4 py-2 text-sm sm:col-span-2" disabled={loading === "zotero"}>{loading === "zotero" ? t("Importiert...") : t("Lokale Bibliothek importieren")}</button>
    </form>
  {:else if activeService === "notion"}
    <form class="surface-panel grid gap-3 p-4 sm:grid-cols-2" onsubmit={(event) => { event.preventDefault(); afterImport("Notion", () => api.importNotionDataSource(notionToken, notionDataSourceId, notionDeckName, notionFrontProperty, notionBackProperty)); }}>
      <label class="text-xs text-secondary">{t("Zielstapel")}<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={notionDeckName} required /></label>
      <label class="text-xs text-secondary">Datenquellen-ID<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={notionDataSourceId} required /></label>
      <label class="text-xs text-secondary">Vorderseiten-Feld<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={notionFrontProperty} required /></label>
      <label class="text-xs text-secondary">Rückseiten-Feld<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={notionBackProperty} required /></label>
      <label class="text-xs text-secondary sm:col-span-2">Integration-Token<input type="password" autocomplete="off" class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={notionToken} required /></label>
      <button class="primary-action w-fit px-4 py-2 text-sm sm:col-span-2" disabled={loading === "notion"}>{loading === "notion" ? t("Importiert...") : t("Datenquelle importieren")}</button>
    </form>
  {:else if activeService === "moodle"}
    <form class="surface-panel grid gap-3 p-4 sm:grid-cols-2" onsubmit={(event) => { event.preventDefault(); afterImport("Moodle", () => api.importMoodleGlossary(moodleBaseUrl, moodleToken, moodleGlossaryId, moodleDeckName)); }}>
      <label class="text-xs text-secondary">{t("Zielstapel")}<input class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={moodleDeckName} required /></label>
      <label class="text-xs text-secondary">Glossar-Aktivitäts-ID<input type="number" min="1" class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={moodleGlossaryId} required /></label>
      <label class="text-xs text-secondary sm:col-span-2">Moodle-URL<input type="url" class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={moodleBaseUrl} placeholder="https://moodle.beispiel.de" required /></label>
      <label class="text-xs text-secondary sm:col-span-2">Webservice-Token<input type="password" autocomplete="off" class="mt-1 w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark" bind:value={moodleToken} required /></label>
      <button class="primary-action w-fit px-4 py-2 text-sm sm:col-span-2" disabled={loading === "moodle"}>{loading === "moodle" ? t("Importiert...") : t("Glossar importieren")}</button>
    </form>
  {/if}
</div>
