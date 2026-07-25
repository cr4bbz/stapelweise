<script lang="ts">
  import { ArchiveRestore, ChevronDown, ChevronUp, Eye, Pencil, Search } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import type { Deck, Exam } from "$lib/types";

  let {
    refreshToken = 0,
    hiddenDecks = [],
    hiddenExams = [],
    onRestore = (_deck: Deck) => {},
    onShowDeck = (_deck: Deck) => {},
    onRestoreExam = (_exam: Exam) => {},
    onShowExam = (_exam: Exam) => {},
    onEditExam = (_exam: Exam) => {},
  } = $props<{
    refreshToken?: number;
    hiddenDecks?: Deck[];
    hiddenExams?: Exam[];
    onRestore?: (deck: Deck) => void;
    onShowDeck?: (deck: Deck) => void;
    onRestoreExam?: (exam: Exam) => void;
    onShowExam?: (exam: Exam) => void;
    onEditExam?: (exam: Exam) => void;
  }>();

  let archivedDecks = $state<Deck[]>([]);
  let archivedExams = $state<Exam[]>([]);
  let loadedToken = $state(-1);
  let expanded = $state(false);
  let archiveQuery = $state("");
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));
  let managedItemCount = $derived(archivedDecks.length + hiddenDecks.length + archivedExams.length + hiddenExams.length);
  let canExpand = $derived(managedItemCount > 4);
  let normalizedArchiveQuery = $derived(archiveQuery.trim().toLocaleLowerCase());
  let filteredArchivedDecks = $derived(archivedDecks.filter((deck: Deck) => deck.name.toLocaleLowerCase().includes(normalizedArchiveQuery)));
  let filteredArchivedExams = $derived(archivedExams.filter((exam: Exam) => exam.name.toLocaleLowerCase().includes(normalizedArchiveQuery)));
  let filteredHiddenDecks = $derived(hiddenDecks.filter((deck: Deck) => deck.name.toLocaleLowerCase().includes(normalizedArchiveQuery)));
  let filteredHiddenExams = $derived(hiddenExams.filter((exam: Exam) => exam.name.toLocaleLowerCase().includes(normalizedArchiveQuery)));
  let hasSearchResults = $derived(filteredArchivedDecks.length + filteredArchivedExams.length + filteredHiddenDecks.length + filteredHiddenExams.length > 0);

  function isPastExam(exam: Exam) {
    const today = new Date();
    const localDate = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
    return exam.exam_date < localDate;
  }

  $effect(() => {
    if (loadedToken === refreshToken) return;
    loadedToken = refreshToken;
    Promise.all([api.listDecks(true), api.listExams(true)])
      .then(([decks, exams]) => {
        archivedDecks = decks.filter((deck) => deck.archived);
        archivedExams = exams.filter((exam) => exam.archived);
      })
      .catch(() => {
        archivedDecks = [];
        archivedExams = [];
      });
  });
</script>

<aside class="surface-panel flex h-full max-h-[30rem] min-h-56 flex-col overflow-hidden p-4">
  <div class="mb-4 flex items-center justify-between gap-3">
    <div>
      <p class="section-kicker">{t("Archiv")}</p>
    </div>
  </div>

  {#if managedItemCount > 0}
    <label class="relative mb-3 block">
      <Search class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-secondary" size={14} aria-hidden="true" />
      <input
        bind:value={archiveQuery}
        type="search"
        class="module-accent-input w-full rounded-md py-2 pl-8 pr-8 text-xs outline-none"
        placeholder={t("Archiv durchsuchen")}
        aria-label={t("Archiv durchsuchen")}
      />
    </label>
  {/if}

  <div class="archive-list min-h-0 pr-1" class:archive-list-expanded={expanded}>
    {#if filteredArchivedDecks.length > 0}
      <div class="grid grid-cols-3 gap-2">
        {#each filteredArchivedDecks as deck (deck.id)}
          <div
            class="module-accent-subpanel flex aspect-[5/3] min-w-0 cursor-pointer items-start overflow-hidden rounded-lg p-2.5 transition-colors hover:border-accent-correct/45"
            role="button"
            tabindex="0"
            aria-label={`${t("Stapel wiederherstellen")}: ${deck.name}`}
            onclick={() => onRestore(deck)}
            onkeydown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onRestore(deck);
              }
            }}
          >
            <h2 data-user-content class="{cardFontClass} line-clamp-3 text-[11px] font-normal leading-tight text-primary dark:text-primary-dark">{deck.name}</h2>
          </div>
        {/each}
      </div>
    {/if}

  {#if filteredArchivedExams.length > 0}
    <div class="space-y-2 {filteredArchivedDecks.length > 0 ? 'mt-2' : ''}">
      {#each filteredArchivedExams as exam (exam.id)}
        <div class="module-accent-subpanel flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
          <div class="min-w-0">
            <p data-user-content class="truncate text-sm font-semibold text-primary dark:text-primary-dark">{exam.name}</p>
            <p class="mt-0.5 text-xs text-secondary">{t("Pr\u00fcfung")}</p>
          </div>
          <div class="flex shrink-0 gap-1">
            {#if isPastExam(exam)}
              <button
                class="icon-button !h-8 !w-8"
                onclick={() => onEditExam(exam)}
                title={t("Pr\u00fcfung bearbeiten")}
                aria-label={t("Pr\u00fcfung bearbeiten")}
              ><Pencil size={15} /></button>
            {:else}
              <button
                class="icon-button !h-8 !w-8"
                onclick={() => onRestoreExam(exam)}
                title={t("Pr\u00fcfung wiederherstellen")}
                aria-label={t("Pr\u00fcfung wiederherstellen")}
              ><ArchiveRestore size={16} /></button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if filteredHiddenDecks.length > 0}
    <p class="mb-2 {filteredArchivedDecks.length > 0 || filteredArchivedExams.length > 0 ? 'mt-5' : ''} text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Stapel")}</p>
    <p class="mb-3 text-xs text-secondary">{t("Ausgeblendete Elemente sind nur vom Dashboard entfernt.")}</p>
    <div class="space-y-2">
      {#each filteredHiddenDecks as deck (deck.id)}
        <div class="module-accent-subpanel flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
          <p data-user-content class="min-w-0 truncate text-sm font-semibold text-primary dark:text-primary-dark">{deck.name}</p>
          <button
            class="icon-button shrink-0 !h-8 !w-8"
            onclick={() => onShowDeck(deck)}
            title={t("Auf Dashboard anzeigen")}
            aria-label={t("Auf Dashboard anzeigen")}
          ><Eye size={16} /></button>
        </div>
      {/each}
    </div>
  {/if}

  {#if filteredHiddenExams.length > 0}
    <p class="mb-2 {filteredHiddenDecks.length > 0 || filteredArchivedDecks.length > 0 || filteredArchivedExams.length > 0 ? 'mt-5' : ''} text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Pr\u00fcfungen")}</p>
    <div class="space-y-2">
      {#each filteredHiddenExams as exam (exam.id)}
        <div class="module-accent-subpanel flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
          <div class="min-w-0">
            <p data-user-content class="truncate text-sm font-semibold text-primary dark:text-primary-dark">{exam.name}</p>
            <p class="mt-0.5 text-xs text-secondary">{exam.exam_date}</p>
          </div>
          <button
            class="icon-button shrink-0 !h-8 !w-8"
            onclick={() => onShowExam(exam)}
            title={t("Pr\u00fcfung auf Dashboard anzeigen")}
            aria-label={t("Pr\u00fcfung auf Dashboard anzeigen")}
          ><Eye size={16} /></button>
        </div>
      {/each}
    </div>
  {/if}

    {#if !hasSearchResults}
      <p class="py-4 text-sm text-secondary">{archiveQuery.trim() ? t("Keine passenden Elemente.") : t("Keine archivierten oder ausgeblendeten Elemente.")}</p>
    {/if}
  </div>
  {#if canExpand}
    <footer class="module-list-footer mt-auto min-h-11 border-t border-current/10 pt-3">
      <button
        type="button"
        class="secondary-action flex h-8 w-full shrink-0 items-center justify-center gap-1.5 !rounded-md px-3 py-0 text-xs"
        onclick={() => (expanded = !expanded)}
        aria-expanded={expanded}
      >
        {#if expanded}
          <ChevronUp size={14} aria-hidden="true" />
          {t("Archiv einklappen")}
        {:else}
          <ChevronDown size={14} aria-hidden="true" />
          {t("Archiv ausklappen")} ({managedItemCount})
        {/if}
      </button>
    </footer>
  {/if}
</aside>

<style>
  .archive-list {
    flex: 1 1 auto;
    min-height: 0;
    overflow: hidden;
    scrollbar-gutter: stable;
  }

  .archive-list-expanded {
    overflow-y: auto;
  }
</style>
