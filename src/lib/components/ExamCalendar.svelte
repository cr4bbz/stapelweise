<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { t } from "$lib/i18n";
  import type { Exam } from "$lib/types";

  let {
    open = false,
    exam = null,
    onClose = () => {},
    onChanged = () => {},
  } = $props<{
    open?: boolean;
    exam?: Exam | null;
    onClose?: () => void;
    onChanged?: () => void;
  }>();

  let name = $state("");
  let examType = $state(t("Klausur"));
  let examDate = $state("");
  let selectedDeckIds = $state<Set<string>>(new Set());
  let submitting = $state(false);

  $effect(() => {
    if (!open) return;
    name = exam?.name ?? "";
    examType = exam?.exam_type ?? t("Klausur");
    examDate = exam?.exam_date ?? "";
    selectedDeckIds = new Set(exam?.deck_ids ?? []);
  });

  function toggleDeck(deckId: string) {
    const next = new Set(selectedDeckIds);
    if (next.has(deckId)) next.delete(deckId);
    else next.add(deckId);
    selectedDeckIds = next;
  }

  async function save() {
    if (!name.trim() || !examDate || selectedDeckIds.size === 0 || submitting) return;
    submitting = true;
    try {
      if (exam) await api.updateExam(exam.id, name.trim(), examType.trim() || t("Klausur"), examDate, [...selectedDeckIds]);
      else await api.createExam(name.trim(), examType.trim() || t("Klausur"), examDate, [...selectedDeckIds]);
      onChanged();
      onClose();
    } catch {
      alert(t("Pr\u00fcfung konnte nicht gespeichert werden."));
    } finally {
      submitting = false;
    }
  }

  async function remove() {
    if (!exam || !confirm(t("Pr\u00fcfung wirklich l\u00f6schen?"))) return;
    await api.deleteExam(exam.id);
    onChanged();
    onClose();
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 backdrop-blur-sm" role="presentation" onclick={onClose} onkeydown={(event) => event.key === "Escape" && onClose()} tabindex="-1">
    <div class="surface-panel max-h-full w-full max-w-2xl overflow-y-auto p-5 sm:p-6" role="dialog" aria-modal="true" aria-label={exam ? t("Pr\u00fcfung bearbeiten") : t("Pr\u00fcfung planen")} tabindex="-1" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
      <div class="mb-5 flex items-center justify-between gap-3">
        <div>
          <p class="section-kicker">{t("Pr\u00fcfung")}</p>
          <h2 class="mt-1 text-xl text-primary dark:text-primary-dark">{exam ? t("Pr\u00fcfung bearbeiten") : t("Pr\u00fcfung planen")}</h2>
        </div>
        <button class="icon-button !h-9 !w-9" onclick={onClose} aria-label={t("Abbrechen")} title={t("Abbrechen")}>x</button>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <label class="min-w-0 text-sm font-medium text-primary dark:text-primary-dark">
          {t("Name")}
          <input bind:value={name} class="module-accent-input mt-1.5 w-full rounded-md px-3 py-2 text-sm outline-none" />
        </label>
        <div class="grid grid-cols-2 gap-3">
          <label class="min-w-0 text-sm font-medium text-primary dark:text-primary-dark">
            {t("Typ")}
            <input bind:value={examType} class="module-accent-input mt-1.5 w-full rounded-md px-3 py-2 text-sm outline-none" />
          </label>
          <label class="min-w-0 text-sm font-medium text-primary dark:text-primary-dark">
            {t("Datum")}
            <input type="date" bind:value={examDate} class="module-accent-input mt-1.5 w-full rounded-md px-3 py-2 text-sm outline-none" />
          </label>
        </div>
      </div>

      <div class="mt-5">
        <p class="text-sm font-medium text-primary dark:text-primary-dark">{t("Relevante Stapel")}</p>
        <div class="mt-2 flex flex-wrap gap-2">
          {#each deckStore.decks as deck}
            <button onclick={() => toggleDeck(deck.id)} class="rounded-md px-3 py-1.5 text-xs font-medium {selectedDeckIds.has(deck.id) ? 'module-accent-selected' : 'module-accent-subpanel text-primary dark:text-primary-dark'}">{deck.name}</button>
          {/each}
        </div>
      </div>

      <div class="mt-6 flex flex-wrap items-center justify-between gap-3">
        {#if exam}
          <button onclick={remove} class="secondary-action px-3 py-2 text-xs text-accent-incorrect">{t("L\u00f6schen")}</button>
        {:else}<span></span>{/if}
        <div class="flex gap-2">
          <button onclick={onClose} class="secondary-action px-3 py-2 text-xs">{t("Abbrechen")}</button>
          <button onclick={save} disabled={!name.trim() || !examDate || selectedDeckIds.size === 0 || submitting} class="primary-action px-4 py-2 text-xs disabled:opacity-45">{exam ? t("\u00c4nderungen speichern") : t("Pr\u00fcfung speichern")}</button>
        </div>
      </div>
    </div>
  </div>
{/if}
