<script lang="ts">
  import * as api from "$lib/api";
  import type { Card, Deck } from "$lib/types";
  import EmptyState from "./EmptyState.svelte";

  let { deck, onClose = () => {} } = $props<{
    deck: Deck;
    onClose?: () => void;
  }>();

  let cards = $state<Card[]>([]);
  let loading = $state(true);
  let showNewCard = $state(false);
  let front = $state("");
  let back = $state("");
  let editingCard = $state<Card | null>(null);

  async function loadCards() {
    loading = true;
    try {
      cards = await api.listCards(deck.id);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadCards();
  });

  async function handleCreate() {
    if (!front.trim() || !back.trim()) return;
    const card = await api.createCard(deck.id, front.trim(), back.trim());
    cards = [card, ...cards];
    front = "";
    back = "";
    showNewCard = false;
  }

  async function handleUpdate() {
    if (!editingCard || !front.trim() || !back.trim()) return;
    await api.updateCard(editingCard.id, front.trim(), back.trim());
    cards = cards.map((c) =>
      c.id === editingCard.id ? { ...c, front: front.trim(), back: back.trim() } : c
    );
    editingCard = null;
    front = "";
    back = "";
  }

  async function handleDelete(cardId: string) {
    await api.deleteCard(cardId);
    cards = cards.filter((c) => c.id !== cardId);
  }

  function startEdit(card: Card) {
    editingCard = card;
    front = card.front;
    back = card.back;
    showNewCard = false;
  }

  function cancelEdit() {
    editingCard = null;
    front = "";
    back = "";
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (editingCard || showNewCard) {
        cancelEdit();
        showNewCard = false;
      } else {
        onClose();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center gap-3 p-6 pb-4">
    <button
      onclick={onClose}
      class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
      title="Zurück zur Übersicht"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark truncate">
      {deck.name}
    </h1>
    <span class="text-secondary text-sm ml-auto">{cards.length} Karten</span>
    <button
      onclick={() => {
        showNewCard = true;
        editingCard = null;
        front = "";
        back = "";
      }}
      class="rounded-button bg-[#1E3A5F] dark:bg-[#E0E0E0] dark:text-[#1A1A2E] text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
    >
      + Neue Karte
    </button>
  </div>

  <!-- Card Form (New or Edit) -->
  {#if showNewCard || editingCard}
    <div class="px-6 pb-4">
      <div class="glass rounded-card p-4 space-y-3">
        <h3 class="text-sm font-medium text-secondary">
          {editingCard ? "Karte bearbeiten" : "Neue Karteikarte"}
        </h3>
        <textarea
          bind:value={front}
          placeholder="Vorderseite (Frage)..."
          class="w-full bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-[#E6A817] transition-colors"
          rows="2"
          autofocus
        ></textarea>
        <textarea
          bind:value={back}
          placeholder="Rückseite (Antwort)..."
          class="w-full bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-[#E6A817] transition-colors"
          rows="2"
        ></textarea>
        <div class="flex gap-2 justify-end">
          <button
            onclick={cancelEdit}
            class="text-secondary text-sm hover:text-primary dark:hover:text-primary-dark"
          >
            Abbrechen
          </button>
          {#if editingCard}
            <button
              onclick={handleUpdate}
              disabled={!front.trim() || !back.trim()}
              class="rounded-button bg-[#E6A817] text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
            >
              Speichern
            </button>
          {:else}
            <button
              onclick={handleCreate}
              disabled={!front.trim() || !back.trim()}
              class="rounded-button bg-[#E6A817] text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
            >
              Erstellen
            </button>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Card List -->
  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <p class="text-secondary">Lädt...</p>
    </div>
  {:else if cards.length === 0}
    <div class="flex-1 flex items-center justify-center">
      <EmptyState
        title="Noch keine Karten"
        description="Erstelle deine erste Karteikarte in diesem Stapel."
        actionLabel="Erste Karte erstellen"
        onAction={() => {
          showNewCard = true;
          editingCard = null;
          front = "";
          back = "";
        }}
        icon={() => "🃏"}
      />
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto px-6 pb-6">
      <div class="space-y-2">
        {#each cards as card (card.id)}
          <div
            class="glass rounded-card p-4 flex items-start justify-between gap-4 group"
          >
            <div class="flex-1 min-w-0 grid grid-cols-2 gap-4">
              <div>
                <span class="text-xs font-medium text-secondary uppercase tracking-wide">Frage</span>
                <p class="text-primary dark:text-primary-dark mt-0.5 line-clamp-3">{card.front}</p>
              </div>
              <div>
                <span class="text-xs font-medium text-secondary uppercase tracking-wide">Antwort</span>
                <p class="text-primary dark:text-primary-dark mt-0.5 line-clamp-3">{card.back}</p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
              <button
                class="p-1.5 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary"
                title="Karte bearbeiten"
                onclick={() => startEdit(card)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                </svg>
              </button>
              <button
                class="p-1.5 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-[#DC2626]"
                title="Karte löschen"
                onclick={() => handleDelete(card.id)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
