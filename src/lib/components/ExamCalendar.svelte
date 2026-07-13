<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { fade, slide } from "svelte/transition";

  let {
    onStudyExam = (_deckIds: string[], _examName: string) => {},
    onSimulateExam = (_deckIds: string[], _examName: string) => {},
  } = $props<{
    onStudyExam?: (deckIds: string[], examName: string) => void;
    onSimulateExam?: (deckIds: string[], examName: string) => void;
  }>();

  let exams = $state<any[]>([]);
  let examStats = $state<Record<string, any>>({});
  
  let editingExamId = $state<string | null>(null);
  let showNewForm = $state(false);
  let newName = $state("");
  let newType = $state("Klausur");
  let newDate = $state("");
  let newDeckIds = $state<Set<string>>(new Set());

  const examTypes = ["Klausur", "Mündliche Prüfung", "Abitur", "Test", "Referat"];

  async function loadExams() {
    try {
      exams = await api.listExams();
      for (const exam of exams) {
        examStats[exam.id] = await api.getExamStats(exam.id);
      }
    } catch (e) {
      console.error("Failed to load exams", e);
    }
  }

  onMount(() => {
    loadExams();
  });

  $effect(() => {
    const openNewExam = () => {
      resetForm();
      showNewForm = true;
    };
    window.addEventListener("stapelweise:new-exam", openNewExam);
    return () => window.removeEventListener("stapelweise:new-exam", openNewExam);
  });

  function startEdit(exam: any) {
    editingExamId = exam.id;
    newName = exam.name;
    newType = exam.exam_type;
    newDate = exam.exam_date;
    newDeckIds = new Set(exam.deck_ids);
    showNewForm = true;
  }

  function resetForm() {
    showNewForm = false;
    editingExamId = null;
    newName = "";
    newType = "Klausur";
    newDate = "";
    newDeckIds = new Set();
  }

  async function handleCreateOrUpdate() {
    if (!newName || !newDate || newDeckIds.size === 0) return;
    try {
      if (editingExamId) {
        await api.updateExam(editingExamId, newName, newType, newDate, Array.from(newDeckIds));
      } else {
        await api.createExam(newName, newType, newDate, Array.from(newDeckIds));
      }
      resetForm();
      await loadExams();
    } catch (e) {
      console.error(e);
      alert("Fehler beim Speichern der Prüfung: " + (e?.toString() || JSON.stringify(e)));
    }
  }

  async function handleDelete(id: string) {
    if (confirm("Prüfung wirklich löschen?")) {
      await api.deleteExam(id);
      await loadExams();
    }
  }

  function urgencyTone(daysLeft: number | undefined): string {
    if (daysLeft === undefined) return "border-[#E4E7EC] bg-[#F8FAFC] text-secondary dark:border-[#2A303B] dark:bg-[#10131A]";
    if (daysLeft <= 3) return "border-red-200 bg-red-50 text-red-700 dark:border-red-500/30 dark:bg-red-500/10 dark:text-red-300";
    if (daysLeft <= 14) return "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-500/30 dark:bg-amber-500/10 dark:text-amber-300";
    return "border-blue-200 bg-blue-50 text-blue-700 dark:border-blue-500/30 dark:bg-blue-500/10 dark:text-blue-300";
  }

  function urgencyLabel(daysLeft: number | undefined): string {
    if (daysLeft === undefined) return "wird berechnet";
    if (daysLeft < 0) return "ueberfaellig";
    if (daysLeft === 0) return "heute";
    if (daysLeft <= 3) return "dringend";
    if (daysLeft <= 14) return "diese Woche";
    return "geplant";
  }

  let customExamTypes = $derived(Array.from(new Set(exams.map(e => e.exam_type))));
  let allExamTypes = $derived(Array.from(new Set([...examTypes, ...customExamTypes])));
</script>

<div class="pb-6">
  <div class="flex flex-col gap-3 mb-4 sm:flex-row sm:items-center sm:justify-between">
    <h2 class="section-heading">
      Prüfungen & Ziele
    </h2>
    <button
      onclick={() => (showNewForm = !showNewForm)}
      class="{showNewForm ? 'secondary-action' : 'primary-action'} px-3 py-1.5 text-xs"
    >
      {showNewForm ? "Abbrechen" : "+ Neue Prüfung"}
    </button>
  </div>

  {#if showNewForm}
    <div transition:slide class="surface-panel p-4 mb-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="exam_new_name" class="block text-xs font-bold text-secondary uppercase tracking-wider mb-1">Name</label>
          <input
            id="exam_new_name"
            type="text"
            bind:value={newName}
            placeholder="z.B. Analysis I"
            class="w-full bg-white dark:bg-[#10131A] border border-[#E4E7EC] dark:border-[#2A303B] rounded-lg px-3 py-2 text-sm text-primary dark:text-primary-dark outline-none focus:border-accent-correct"
          />
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="exam_new_type" class="block text-xs font-bold text-secondary uppercase tracking-wider mb-1">Typ</label>
            <input
              id="exam_new_type"
              type="text"
              list="exam-types-list"
              bind:value={newType}
              placeholder="z.B. Klausur"
              class="w-full bg-white dark:bg-[#10131A] border border-[#E4E7EC] dark:border-[#2A303B] rounded-lg px-3 py-2 text-sm text-primary dark:text-primary-dark outline-none focus:border-accent-correct"
            />
            <datalist id="exam-types-list">
              {#each allExamTypes as t}
                <option value={t}></option>
              {/each}
            </datalist>
          </div>
          <div>
            <label for="exam_new_date" class="block text-xs font-bold text-secondary uppercase tracking-wider mb-1">Datum</label>
            <input
              id="exam_new_date"
              type="date"
              bind:value={newDate}
              class="w-full bg-white dark:bg-[#10131A] border border-[#E4E7EC] dark:border-[#2A303B] rounded-lg px-3 py-2 text-sm text-primary dark:text-primary-dark outline-none focus:border-accent-correct cursor-pointer"
            />
          </div>
        </div>
      </div>

      <div class="mt-4">
        <span class="block text-xs font-bold text-secondary uppercase tracking-wider mb-2">Relevante Stapel</span>
        <div class="flex flex-wrap gap-2">
          {#each deckStore.decks as deck}
            <button
              onclick={() => {
                const next = new Set(newDeckIds);
                if (next.has(deck.id)) next.delete(deck.id);
                else next.add(deck.id);
                newDeckIds = next;
              }}
              class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-all {newDeckIds.has(deck.id) ? 'bg-accent-correct border-accent-correct text-white' : 'bg-white dark:bg-[#171B24] border-[#E4E7EC] dark:border-[#2A303B] text-primary dark:text-primary-dark'}"
            >
              {deck.name}
            </button>
          {/each}
        </div>
      </div>

      <div class="mt-5 flex justify-end gap-2">
        {#if editingExamId}
          <button
            onclick={resetForm}
            class="px-4 py-2 text-xs font-medium text-secondary hover:text-primary dark:hover:text-primary-dark"
          >
            Abbrechen
          </button>
        {/if}
        <button
          onclick={handleCreateOrUpdate}
          disabled={!newName || !newDate || newDeckIds.size === 0}
          class="primary-action px-5 py-2 text-sm disabled:opacity-50"
        >
          {editingExamId ? "Änderungen speichern" : "Prüfung speichern"}
        </button>
      </div>
    </div>
  {/if}

  {#if exams.length === 0 && !showNewForm}
    <div class="text-sm text-secondary text-center py-4 surface-panel">
      Keine anstehenden Prüfungen eingetragen.
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each exams as exam}
        {@const stats = examStats[exam.id]}
        <div class="surface-panel p-4 relative group">
          <div class="absolute top-3 right-3 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onclick={() => startEdit(exam)}
              class="p-1 rounded text-secondary hover:text-primary dark:hover:text-primary-dark hover:bg-white/10 transition-colors"
              title="Bearbeiten"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
              </svg>
            </button>
            <button
              onclick={() => handleDelete(exam.id)}
              class="p-1 rounded text-secondary hover:text-accent-incorrect hover:bg-white/10 transition-colors"
              title="Löschen"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>

          <div class="flex items-start gap-3">
            <div class="min-w-[5.5rem] rounded-xl border px-3 py-2 text-center shadow-sm {urgencyTone(stats?.days_left)}">
              <span class="mx-auto mb-1 inline-flex max-w-full items-center rounded-md bg-white/70 px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-current dark:bg-black/20">
                {exam.exam_type}
              </span>
              <div class="flex items-baseline justify-center gap-1">
                <span class="font-pixel text-xl font-black leading-none">{stats ? Math.max(0, stats.days_left) : '-'}</span>
                <span class="text-[10px] font-bold uppercase tracking-wide">Tage</span>
              </div>
              <span class="mt-1 block text-[10px] font-semibold uppercase tracking-wide opacity-80">
                {urgencyLabel(stats?.days_left)}
              </span>
            </div>
            <div class="min-w-0 pt-1">
              <h3 class="truncate pr-12 font-bold text-primary dark:text-primary-dark">{exam.name}</h3>
              <p class="text-xs text-secondary mt-0.5">{new Date(exam.exam_date).toLocaleDateString('de-DE')}</p>
            </div>
          </div>

          {#if stats}
            <div class="mt-4 pt-3 border-t border-white/10 dark:border-white/5 flex items-center justify-between">
              <div>
                <p class="text-xs text-secondary mb-1">Lernziel</p>
                <p class="font-bold text-accent-correct text-lg">{stats.cards_per_day} <span class="text-xs font-medium text-primary dark:text-primary-dark">Karten / Tag</span></p>
              </div>
              <div class="text-right">
                <p class="text-xs text-secondary mb-1">Rest</p>
                <p class="font-medium text-primary dark:text-primary-dark text-sm">{stats.cards_left} / {stats.total_cards}</p>
              </div>
            </div>
          {/if}

          <div class="mt-3 flex gap-2">
            <button
              onclick={() => onStudyExam(exam.deck_ids, exam.name)}
              class="flex-1 py-1.5 primary-action text-xs"
            >
              Lernen
            </button>
            <button
              onclick={() => onSimulateExam(exam.deck_ids, exam.name)}
              class="px-3 py-1.5 secondary-action text-xs"
              title="Test-Simulation starten"
            >
              Simulation
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
