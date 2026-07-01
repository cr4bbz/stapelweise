<script lang="ts">
  import { deckStore } from "$lib/stores/decks.svelte";
  import { studyStore } from "$lib/stores/study.svelte";
  import EmptyState from "./EmptyState.svelte";
  import ErrorBanner from "./ErrorBanner.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import type { Deck } from "$lib/types";

  let {
    onSelectDeck = (_deck: Deck) => {},
    onStudyDeck = (_deck: Deck) => {},
    onStudyDecks = (_decks: Deck[]) => {},
  } = $props<{
    onSelectDeck?: (deck: Deck) => void;
    onStudyDeck?: (deck: Deck) => void;
    onStudyDecks?: (decks: Deck[]) => void;
  }>();

  let showNewDeck = $state(false);
  let newDeckName = $state("");
  let editingDeckId = $state<string | null>(null);
  let editingName = $state("");
  let dueCounts = $state<Record<string, number>>({});
  let totalCounts = $state<Record<string, number>>({});
  let error = $state<string | null>(null);
  let deleteConfirmDeck = $state<Deck | null>(null);
  let selectedDeckIds = $state<Set<string>>(new Set());

  const store = deckStore;

  async function loadCounts() {
    const missing = store.decks.filter(d => !(d.id in totalCounts));
    if (missing.length === 0) return;
    await Promise.all(missing.map(async (deck) => {
      try {
        await studyStore.loadCounts(deck.id);
        dueCounts = { ...dueCounts, [deck.id]: studyStore.dueCount };
        totalCounts = { ...totalCounts, [deck.id]: studyStore.totalCount };
      } catch {
        // If counting fails, it's fine — the deck might be new
      }
    }));
  }

  $effect(() => {
    if (store.decks.length > 0) {
      loadCounts();
    }
  });

  async function handleCreate() {
    const name = newDeckName.trim();
    if (!name) return;
    error = null;
    try {
      const deck = await store.create(name);
      newDeckName = "";
      showNewDeck = false;
      // Auto-open the new deck so user can add cards
      onSelectDeck(deck);
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Erstellen des Stapels";
    }
  }

  function startEdit(deck: Deck) {
    editingDeckId = deck.id;
    editingName = deck.name;
  }

  async function handleRename() {
    if (!editingDeckId || !editingName.trim()) return;
    error = null;
    try {
      await store.update(editingDeckId, editingName.trim());
      editingDeckId = null;
      editingName = "";
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Umbenennen";
    }
  }

  async function handleDelete(deck: Deck) {
    error = null;
    try {
      await store.remove(deck.id);
      const { [deck.id]: _, ...rest } = dueCounts;
      dueCounts = rest;
      const { [deck.id]: __, ...rest2 } = totalCounts;
      totalCounts = rest2;
      if (selectedDeckIds.has(deck.id)) {
        const next = new Set(selectedDeckIds);
        next.delete(deck.id);
        selectedDeckIds = next;
      }
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Löschen";
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showNewDeck = false;
      editingDeckId = null;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between p-6 pb-4">
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark">
      Deine Stapel
    </h1>
    <div class="flex gap-2">
      {#if selectedDeckIds.size > 0}
        <button
          onclick={() => {
            const decks = store.decks.filter((d) => selectedDeckIds.has(d.id));
            selectedDeckIds = new Set();
            onStudyDecks(decks);
          }}
          class="rounded-button bg-accent-correct text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
        >
          {selectedDeckIds.size} lernen
        </button>
      {/if}
      <button
        onclick={async () => {
          try {
            await store.seed();
            await loadCounts();
          } catch (e: any) {
            error = e?.toString() || "Fehler beim Laden der Beispieldaten";
          }
        }}
        disabled={store.hasSeeded && store.decks.length > 0}
        class="rounded-button bg-accent-correct text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
        title={store.hasSeeded && store.decks.length > 0 ? "Bereits geladen" : "3 thematische Decks mit 18 Karten in verschiedenen Lernzuständen"}
      >
        {store.hasSeeded && store.decks.length > 0 ? "Geladen" : "Beispieldaten"}
      </button>
      <button
        onclick={() => (showNewDeck = true)}
        class="rounded-button bg-primary dark:bg-[#E0E0E0] dark:text-[#1A1A2E] text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
      >
        + Neuer Stapel
      </button>
    </div>
  </div>

  <!-- Error -->
  {#if error}
    <ErrorBanner message={error} onDismiss={() => (error = null)} />
  {/if}

  <!-- New Deck Input -->
  {#if showNewDeck}
    <div class="px-6 pb-4">
      <div class="glass rounded-card p-4 flex gap-3 items-center">
        <input
          type="text"
          bind:value={newDeckName}
          placeholder="Name des Stapels..."
          class="flex-1 bg-transparent border-none outline-none text-lg text-primary dark:text-primary-dark placeholder:text-secondary"
          onkeydown={(e) => {
            if (e.key === "Enter") handleCreate();
            if (e.key === "Escape") showNewDeck = false;
          }}
          autofocus
        />
        <button
          onclick={handleCreate}
          disabled={!newDeckName.trim()}
          class="rounded-button bg-accent-correct text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
        >
          Anlegen
        </button>
        <button
          onclick={() => (showNewDeck = false)}
          class="text-secondary hover:text-primary dark:hover:text-primary-dark text-sm"
        >
          Abbrechen
        </button>
      </div>
    </div>
  {/if}

  <!-- Deck Grid -->
  {#if store.decks.length === 0 && !showNewDeck}
    <div class="flex-1 flex items-center justify-center">
      <EmptyState
        title="Noch kein Stapel"
        description="Erstelle deinen ersten Karteikarten-Stapel und leg los."
        actionLabel="Ersten Stapel anlegen"
        onAction={() => (showNewDeck = true)}
        icon={() => "🗂️"}
      />
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <div class="grid gap-3">
        {#each store.decks as deck (deck.id)}
          <div class="glass rounded-card p-4 transition-shadow flex items-center gap-3 shadow-elevation-low hover:shadow-elevation-mid">
            {#if (totalCounts[deck.id] ?? 0) > 0}
              <input
                type="checkbox"
                checked={selectedDeckIds.has(deck.id)}
                onchange={() => {
                  const next = new Set(selectedDeckIds);
                  if (next.has(deck.id)) next.delete(deck.id);
                  else next.add(deck.id);
                  selectedDeckIds = next;
                }}
                class="w-5 h-5 accent-accent-correct shrink-0 cursor-pointer"
                title="Zum Lernen auswählen"
              />
            {/if}
            <!-- Deck info (clickable → opens cards) -->
            <button
              class="flex-1 min-w-0 text-left"
              onclick={() => onSelectDeck(deck)}
              title="Karten anzeigen & bearbeiten"
            >
              {#if editingDeckId === deck.id}
                <input
                  type="text"
                  bind:value={editingName}
                  class="w-full bg-transparent border-b-2 border-accent-correct outline-none text-lg font-semibold text-primary dark:text-primary-dark pb-0.5"
                  onkeydown={(e) => {
                    if (e.key === "Enter") handleRename();
                    if (e.key === "Escape") editingDeckId = null;
                  }}
                  autofocus
                />
              {:else}
                <h3 class="text-lg font-semibold text-primary dark:text-primary-dark truncate">
                  {deck.name}
                </h3>
              {/if}
              <p class="text-sm text-secondary mt-1">
                {totalCounts[deck.id] ?? "..."} Karten
                {#if (dueCounts[deck.id] ?? 0) > 0}
                  &middot; <span class="text-accent-correct font-medium">{dueCounts[deck.id]} fällig</span>
                {:else if (totalCounts[deck.id] ?? 0) > 0}
                  &middot; <span class="text-green-600 dark:text-green-400 font-medium">alles klar!</span>
                {/if}
              </p>
            </button>

            <!-- Action buttons (always visible) -->
            <div class="flex items-center gap-0.5 shrink-0">
              {#if editingDeckId === deck.id}
                <button
                  class="px-3 py-1.5 text-sm rounded-button bg-accent-correct text-white font-medium hover:scale-[1.02] transition-transform"
                  onclick={handleRename}
                >Speichern</button>
                <button
                  class="px-3 py-1.5 text-sm text-secondary hover:text-primary dark:hover:text-primary-dark"
                  onclick={() => (editingDeckId = null)}
                >Abbrechen</button>
              {:else}
                <!-- Study button (only if cards exist) -->
                {#if (totalCounts[deck.id] ?? 0) > 0}
                  <button
                    class="px-3 py-2 rounded-lg hover:bg-accent-correct/10 text-accent-correct font-medium text-sm transition-colors"
                    title="Stapel lernen"
                    onclick={(e) => {
                      e.stopPropagation();
                      onStudyDeck(deck);
                    }}
                  >
                    Lernen
                  </button>
                {/if}
                <button
                  class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
                  title="Stapel umbenennen"
                  onclick={() => startEdit(deck)}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                  </svg>
                </button>
                <button
                  class="p-2 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-accent-incorrect transition-colors"
                  title="Stapel löschen"
                  onclick={() => (deleteConfirmDeck = deck)}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

{#if deleteConfirmDeck}
  <ConfirmDialog
    message={'Der Stapel "' + deleteConfirmDeck.name + '" und alle seine Karten werden dauerhaft gelöscht.'}
    confirmLabel="Stapel löschen"
    onConfirm={() => {
      handleDelete(deleteConfirmDeck!);
      deleteConfirmDeck = null;
    }}
    onCancel={() => (deleteConfirmDeck = null)}
  />
{/if}
