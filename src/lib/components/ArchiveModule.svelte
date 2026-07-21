<script lang="ts">
  import { ArchiveRestore, Eye, Pencil } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
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
      <p class="mt-1 text-sm text-secondary">{archivedDecks.length + hiddenDecks.length + archivedExams.length + hiddenExams.length} {t("verwaltete Elemente")}</p>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto pr-1">
    {#if archivedDecks.length > 0}
      <p class="mb-2 text-xs font-semibold uppercase text-secondary">{t("Archiviert")}</p>
      <div class="space-y-2">
        {#each archivedDecks as deck (deck.id)}
          <div class="module-accent-subpanel flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
            <p data-user-content class="min-w-0 truncate text-sm font-semibold text-primary dark:text-primary-dark">{deck.name}</p>
            <button
              class="icon-button shrink-0 !h-8 !w-8"
              onclick={() => onRestore(deck)}
              title={t("Stapel wiederherstellen")}
              aria-label={t("Stapel wiederherstellen")}
            ><ArchiveRestore size={16} /></button>
          </div>
        {/each}
      </div>
    {/if}

  {#if archivedExams.length > 0}
    <div class="space-y-2 {archivedDecks.length > 0 ? 'mt-2' : ''}">
      {#each archivedExams as exam (exam.id)}
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

  {#if hiddenDecks.length > 0}
    <p class="mb-2 {archivedDecks.length > 0 || archivedExams.length > 0 ? 'mt-5' : ''} text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Stapel")}</p>
    <div class="space-y-2">
      {#each hiddenDecks as deck (deck.id)}
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

  {#if hiddenExams.length > 0}
    <p class="mb-2 {hiddenDecks.length > 0 || archivedDecks.length > 0 || archivedExams.length > 0 ? 'mt-5' : ''} text-xs font-semibold uppercase text-secondary">{t("Ausgeblendete Pr\u00fcfungen")}</p>
    <div class="space-y-2">
      {#each hiddenExams as exam (exam.id)}
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

    {#if archivedDecks.length === 0 && hiddenDecks.length === 0 && archivedExams.length === 0 && hiddenExams.length === 0}
      <p class="py-4 text-sm text-secondary">{t("Keine archivierten oder ausgeblendeten Elemente.")}</p>
    {/if}
  </div>
</aside>
