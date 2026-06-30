<script lang="ts">
  import { deckStore } from "$lib/stores/decks.svelte";
  import { studyStore } from "$lib/stores/study.svelte";
  import EmptyState from "./EmptyState.svelte";
  import CardEditor from "./CardEditor.svelte";
  import type { Deck } from "$lib/types";

  let showNewDeck = $state(false);
  let newDeckName = $state("");
  let editingDeck = $state<Deck | null>(null);
  let selectedDeck = $state<Deck | null>(null);
  let dueCounts = $state<Record<string, number>>({});
  let totalCounts = $state<Record<string, number>>({});

  const store = deckStore;

  async function loadCounts() {
    for (const deck of store.decks) {
      studyStore.loadCounts(deck.id).then(() => {
        dueCounts = { ...dueCounts, [deck.id]: studyStore.dueCount };
        totalCounts = { ...totalCounts, [deck.id]: studyStore.totalCount };
      });
    }
  }

  $effect(() => {
    if (store.decks.length > 0) {
      loadCounts();
    }
  });

  async function handleCreate() {
    if (!newDeckName.trim()) return;
    await store.create(newDeckName.trim());
    newDeckName = "";
    showNewDeck = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showNewDeck = false;
      editingDeck = null;
      selectedDeck = null;
    }
  }

  function startStudy(deck: Deck) {
    // Navigate to study view
    window.location.hash = `#/study/${deck.id}`;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if selectedDeck && !editingDeck}
  <CardEditor
    deck={selectedDeck}
    onClose={() => (selectedDeck = null)}
  />
{:else}
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center justify-between p-6 pb-4">
      <h1 class="text-2xl font-bold text-primary dark:text-primary-dark">
        Deine Stapel
      </h1>
      <button
        onclick={() => (showNewDeck = true)}
        class="rounded-button bg-[#1E3A5F] dark:bg-[#E0E0E0] dark:text-[#1A1A2E] text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
      >
        + Neuer Stapel
      </button>
    </div>

    <!-- New Deck Input -->
    {#if showNewDeck}
      <div class="px-6 pb-4">
        <div class="glass rounded-card p-4 flex gap-3 items-center">
          <input
            type="text"
            bind:value={newDeckName}
            placeholder="Name des Stapels..."
            class="flex-1 bg-transparent border-none outline-none text-lg text-primary dark:text-primary-dark placeholder:text-secondary"
            onkeydown={(e) => e.key === "Enter" && handleCreate()}
            autofocus
          />
          <button
            onclick={handleCreate}
            disabled={!newDeckName.trim()}
            class="rounded-button bg-[#E6A817] text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
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
    {#if store.decks.length === 0}
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
            <div
              class="glass rounded-card p-4 cursor-pointer hover:scale-[1.01] transition-all flex items-center justify-between group"
              role="button"
              tabindex="0"
              onclick={() => startStudy(deck)}
              onkeydown={(e) => e.key === "Enter" && startStudy(deck)}
            >
              <div class="flex-1 min-w-0">
                <h3 class="text-lg font-semibold text-primary dark:text-primary-dark truncate">
                  {deck.name}
                </h3>
                <p class="text-sm text-secondary mt-1">
                  {totalCounts[deck.id] ?? "..."} Karten
                  {#if (dueCounts[deck.id] ?? 0) > 0}
                    &middot; <span class="text-[#E6A817] font-medium">{dueCounts[deck.id]} fällig</span>
                  {/if}
                </p>
              </div>

              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary"
                  title="Stapel bearbeiten"
                  onclick={(e) => {
                    e.stopPropagation();
                    editingDeck = deck;
                  }}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                  </svg>
                </button>
                <button
                  class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary"
                  title="Karten anzeigen"
                  onclick={(e) => {
                    e.stopPropagation();
                    selectedDeck = deck;
                  }}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M7 3a1 1 0 000 2h6a1 1 0 100-2H7zM4 7a1 1 0 011-1h10a1 1 0 110 2H5a1 1 0 01-1-1zM2 11a2 2 0 012-2h12a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4z" />
                  </svg>
                </button>
                <button
                  class="p-2 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-[#DC2626]"
                  title="Stapel löschen"
                  onclick={async (e) => {
                    e.stopPropagation();
                    await store.remove(deck.id);
                  }}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}
