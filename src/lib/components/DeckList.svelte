<script lang="ts">
  import { deckStore } from "$lib/stores/decks.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import EmptyState from "./EmptyState.svelte";
  import ErrorBanner from "./ErrorBanner.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import * as api from "$lib/api";
  import type { Deck } from "$lib/types";
  import { tick } from "svelte";
  import { fade } from "svelte/transition";

  let {
    onSelectDeck = (_deck: Deck) => {},
    onStudyDeck = (_deck: Deck) => {},
    onPracticeDeck = (_deck: Deck) => {},
    onStudyDecks = (_decks: Deck[]) => {},
    onLibraryChanged = () => {},
    refreshToken = 0,
  } = $props<{
    onSelectDeck?: (deck: Deck) => void;
    onStudyDeck?: (deck: Deck) => void;
    onPracticeDeck?: (deck: Deck) => void;
    onStudyDecks?: (decks: Deck[]) => void;
    onLibraryChanged?: () => void;
    refreshToken?: number;
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
  let newDeckInput = $state<HTMLInputElement | null>(null);
  let editingDeckInput = $state<HTMLInputElement | null>(null);
  let importInput = $state<HTMLInputElement | null>(null);
  let appliedRefreshToken = -1;

  const store = deckStore;
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  async function loadCounts() {
    const missing = store.decks.filter(d => !(d.id in totalCounts));
    if (missing.length === 0) return;
    await Promise.all(missing.map(async (deck) => {
      try {
        const [due, total] = await Promise.all([
          api.countDueCards(deck.id),
          api.countTotalCards(deck.id),
        ]);
        dueCounts = { ...dueCounts, [deck.id]: due };
        totalCounts = { ...totalCounts, [deck.id]: total };
      } catch {
        // If counting fails, it's fine — the deck might be new
      }
    }));
  }

  $effect(() => {
    settingsStore.load();
    if (refreshToken !== appliedRefreshToken) {
      appliedRefreshToken = refreshToken;
      dueCounts = {};
      totalCounts = {};
    }
    if (store.decks.length > 0) {
      loadCounts();
    }
  });

  $effect(() => {
    if (showNewDeck) {
      tick().then(() => newDeckInput?.focus());
    }
  });

  $effect(() => {
    if (editingDeckId) {
      tick().then(() => editingDeckInput?.focus());
    }
  });

  $effect(() => {
    const openNewDeck = () => {
      showNewDeck = true;
      editingDeckId = null;
    };
    window.addEventListener("stapelweise:new-deck", openNewDeck);
    return () => window.removeEventListener("stapelweise:new-deck", openNewDeck);
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
      onLibraryChanged();
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

  async function handleImport(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const data = JSON.parse(text);
      if (!data.name || !Array.isArray(data.cards)) {
        throw new Error("Ungültiges JSON Format für Deck");
      }
      
      await api.importDeckFromJson(data.name, data.cards);
      
      await store.load();
      await loadCounts();
      onLibraryChanged();
    } catch (err: any) {
      error = "Fehler beim Importieren: " + (err?.toString() || "");
    }
    input.value = ""; // reset input
  }

  function triggerImport() {
    importInput?.click();
  }

  async function handleSeedSamples() {
    error = null;
    try {
      await store.seed();
      await loadCounts();
      onLibraryChanged();
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Laden der Beispieldaten";
    }
  }

  async function handleExport(deck: Deck) {
    try {
      const cards = await api.listCards(deck.id);
      const exportData = {
        name: deck.name,
        cards: cards.map(c => ({
          front: c.front,
          back: c.back,
          front_language: c.front_language,
          back_language: c.back_language,
          reasoning: c.reasoning,
          tags: c.tags,
          card_type: c.card_type,
          content: c.content
        }))
      };
      const json = JSON.stringify(exportData, null, 2);
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${deck.name.replace(/[^a-z0-9\u00c0-\u024f]/gi, '_').toLowerCase()}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (e: any) {
      error = "Fehler beim Exportieren: " + (e?.toString() || "");
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="flex flex-col">
  <!-- Header -->
  <div class="section-header">
    <h2 class="section-heading">
      Deine Stapel
    </h2>
    <div class="flex flex-wrap gap-2">
      {#if selectedDeckIds.size > 0}
        <button
          onclick={() => {
            const decks = store.decks.filter((d) => selectedDeckIds.has(d.id));
            selectedDeckIds = new Set();
            onStudyDecks(decks);
          }}
          class="primary-action px-4 py-2 text-sm"
        >
          {selectedDeckIds.size} lernen
        </button>
      {/if}

      <label class="secondary-action px-4 py-2 text-sm cursor-pointer">
        Import JSON
        <input bind:this={importInput} type="file" accept=".json" class="hidden" onchange={handleImport} />
      </label>
      <button
        onclick={() => (showNewDeck = true)}
        class="primary-action px-4 py-2 text-sm"
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
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="pb-4">
      <div class="surface-panel p-4 flex gap-3 items-center">
        <input
          bind:this={newDeckInput}
          type="text"
          bind:value={newDeckName}
          placeholder="Name des Stapels..."
          class="flex-1 bg-transparent border-none outline-none text-lg text-primary dark:text-primary-dark placeholder:text-secondary"
          onkeydown={(e) => {
            if (e.key === "Enter") handleCreate();
            if (e.key === "Escape") showNewDeck = false;
          }}
        />
        <button
          onclick={handleCreate}
          disabled={!newDeckName.trim()}
          class="primary-action px-4 py-1.5 text-sm disabled:opacity-50"
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
  <div>
    {#if store.decks.length === 0 && !showNewDeck}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="surface-panel flex flex-col items-center justify-center px-5 py-12 text-center">
      <EmptyState
        title="Noch kein Stapel"
        description="Erstelle deinen ersten Stapel, probiere Beispieldaten aus oder importiere vorhandene Karten."
        actionLabel="Ersten Stapel anlegen"
        onAction={() => (showNewDeck = true)}
        icon={() => "🗂️"}
      />
      <div class="-mt-8 flex flex-wrap justify-center gap-2">
        <button class="secondary-action px-4 py-2 text-sm" onclick={handleSeedSamples}>Beispieldaten laden</button>
        <button class="secondary-action px-4 py-2 text-sm" onclick={triggerImport}>JSON importieren</button>
      </div>
      </div>
    {:else}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 pb-12 pt-1">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each store.decks as deck (deck.id)}
          {@const count = totalCounts[deck.id] ?? 0}
          {@const layers = count === 0 ? 0 : Math.min(Math.ceil(count / 5), 12)}
          
          <div class="relative group isolate min-h-44">
            <!-- Dynamic stack layers -->
            {#each Array(layers) as _, i}
              <div 
                class="hidden"
                style="
                  z-index: {-10 - i};
                  transform: translateY({(i + 1) * 3}px) scale({1 - (i + 1) * 0.015});
                  opacity: {Math.max(0.2, 1 - (i * 0.08))};
                "
              ></div>
            {/each}
            
            <div class="surface-panel h-full p-4 transition-all duration-200 flex flex-col hover:border-accent-correct/35 hover:shadow-elevation-mid">
              {#if (totalCounts[deck.id] ?? 0) > 0}
                <div class="absolute top-4 right-4 z-20">
                  <input
                    type="checkbox"
                    checked={selectedDeckIds.has(deck.id)}
                    onchange={() => {
                      const next = new Set(selectedDeckIds);
                      if (next.has(deck.id)) next.delete(deck.id);
                      else next.add(deck.id);
                      selectedDeckIds = next;
                    }}
                    class="w-4 h-4 accent-accent-correct cursor-pointer"
                    title="Zum Lernen auswählen"
                  />
                </div>
              {/if}
              
              <!-- Deck info (clickable → opens cards) -->
              <button
                class="flex-1 flex flex-col items-start w-full text-left outline-none"
                onclick={() => onSelectDeck(deck)}
                title="Karten anzeigen & bearbeiten"
              >
                {#if editingDeckId === deck.id}
                  <input
                    bind:this={editingDeckInput}
                    type="text"
                    bind:value={editingName}
                    class="{cardFontClass} w-[85%] bg-transparent border-b-2 border-accent-correct outline-none text-xl text-primary dark:text-primary-dark pb-0.5"
                    onkeydown={(e) => {
                      if (e.key === "Enter") handleRename();
                      if (e.key === "Escape") editingDeckId = null;
                    }}
                    onclick={(e) => e.stopPropagation()}
                  />
                {:else}
                  <h3 class="{cardFontClass} text-lg text-primary dark:text-primary-dark line-clamp-2 w-[85%] leading-tight">
                    {deck.name}
                  </h3>
                {/if}
                
                <div class="mt-auto pt-5">
                  <p class="text-sm font-medium text-secondary">
                    {totalCounts[deck.id] ?? "..."} Karten
                  </p>
                  {#if (dueCounts[deck.id] ?? 0) > 0}
                    <p class="text-accent-correct font-semibold text-sm">{dueCounts[deck.id]} fällig</p>
                  {:else if (totalCounts[deck.id] ?? 0) > 0}
                    <p class="text-green-600 dark:text-green-400 font-semibold text-sm">alles klar!</p>
                  {/if}
                </div>
              </button>

              <!-- Clear navigation and learning actions -->
              <div class="flex shrink-0 flex-col gap-2 pt-3 mt-3 border-t border-[#E4E7EC] dark:border-[#2A303B] w-full">
                {#if editingDeckId === deck.id}
                  <div class="flex gap-2">
                    <button
                      class="primary-action px-3 py-1.5 text-xs"
                      onclick={handleRename}
                    >Speichern</button>
                    <button
                      class="px-3 py-1.5 text-xs text-secondary hover:text-primary dark:hover:text-primary-dark"
                      onclick={() => (editingDeckId = null)}
                    >Abbrechen</button>
                  </div>
                {:else}
                  <div class="flex items-center justify-between gap-2">
                    <button class="secondary-action px-3 py-1.5 text-xs" onclick={() => onSelectDeck(deck)}>
                      Stapel öffnen
                    </button>
                    <div class="flex gap-0.5">
                      <button class="icon-button !h-8 !w-8" title="Stapel umbenennen" onclick={() => startEdit(deck)}>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" /></svg>
                      </button>
                      <button class="icon-button !h-8 !w-8" title="Stapel exportieren (JSON)" onclick={() => handleExport(deck)}>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" /></svg>
                      </button>
                      <button class="icon-button !h-8 !w-8 hover:!text-accent-incorrect" title="Stapel löschen" onclick={() => (deleteConfirmDeck = deck)}>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" /></svg>
                      </button>
                    </div>
                  </div>
                  {#if (totalCounts[deck.id] ?? 0) > 0}
                    <div class="grid grid-cols-2 gap-2">
                      {#if (dueCounts[deck.id] ?? 0) > 0}
                        <button class="primary-action px-3 py-1.5 text-xs" onclick={() => onStudyDeck(deck)}>
                          Fällige lernen
                        </button>
                      {/if}
                      <button
                        class="{(dueCounts[deck.id] ?? 0) > 0 ? 'secondary-action' : 'primary-action col-span-2'} px-3 py-1.5 text-xs"
                        onclick={() => onPracticeDeck(deck)}
                      >
                        Frei lernen
                      </button>
                    </div>
                  {/if}
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
      </div>
    {/if}
  </div>
</section>

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
