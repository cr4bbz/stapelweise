<script lang="ts">
  import * as api from "$lib/api";
  import type { Card, CardState, Deck } from "$lib/types";
  import EmptyState from "./EmptyState.svelte";
  import ErrorBanner from "./ErrorBanner.svelte";
  import StatsDashboard from "./StatsDashboard.svelte";
  import { renderMath, hasMath } from "$lib/math";

  let { deck, onClose = () => {}, onStudy = () => {} } = $props<{
    deck: Deck;
    onClose?: () => void;
    onStudy?: () => void;
  }>();

  let cards = $state<Card[]>([]);
  let loading = $state(true);
  let showNewCard = $state(false);
  let front = $state("");
  let back = $state("");
  let editingCard = $state<Card | null>(null);
  let error = $state<string | null>(null);
  let dueCount = $state<number | null>(null);
  let showStats = $state(false);

  async function loadCards() {
    loading = true;
    error = null;
    try {
      cards = await api.listCards(deck.id);
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Laden der Karten";
    } finally {
      loading = false;
    }
  }

  async function loadDueCount() {
    try {
      dueCount = await api.countDueCards(deck.id);
    } catch {
      // ignore
    }
  }

  $effect(() => {
    loadCards();
    loadDueCount();
  });

  async function handleCreate() {
    if (!front.trim() || !back.trim()) return;
    error = null;
    try {
      const card = await api.createCard(deck.id, front.trim(), back.trim());
      cards = [card, ...cards];
      front = "";
      back = "";
      showNewCard = false;
      loadDueCount();
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Erstellen der Karte";
    }
  }

  async function handleUpdate() {
    if (!editingCard || !front.trim() || !back.trim()) return;
    error = null;
    try {
      await api.updateCard(editingCard.id, front.trim(), back.trim());
      cards = cards.map((c) =>
        c.id === editingCard.id ? { ...c, front: front.trim(), back: back.trim() } : c
      );
      editingCard = null;
      front = "";
      back = "";
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Speichern der Karte";
    }
  }

  let cardStateOverlay = $state<{card: Card, state: CardState} | null>(null);

  async function toggleCardState(card: Card) {
    if (cardStateOverlay?.card.id === card.id) {
      cardStateOverlay = null;
      return;
    }
    try {
      const state = await api.getCardState(card.id);
      if (state) {
        cardStateOverlay = { card, state };
      }
    } catch {
      // ignore
    }
  }

  async function handleDelete(cardId: string) {
    error = null;
    try {
      await api.deleteCard(cardId);
      cards = cards.filter((c) => c.id !== cardId);
      loadDueCount();
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Löschen der Karte";
    }
  }

  function startEdit(card: Card) {
    editingCard = card;
    front = card.front;
    back = card.back;
    showNewCard = false;
    error = null;
  }

  function cancelEdit() {
    editingCard = null;
    showNewCard = false;
    front = "";
    back = "";
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (editingCard || showNewCard) {
        cancelEdit();
      } else {
        onClose();
      }
    }
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      if (editingCard) handleUpdate();
      else if (showNewCard) handleCreate();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if showStats}
  <StatsDashboard deckId={deck.id} deckName={deck.name} onClose={() => (showStats = false)} />
{:else}
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
    <span class="text-secondary text-sm">{cards.length} Karten</span>
    {#if dueCount != null && dueCount > 0}
      <span class="text-accent-correct text-sm font-medium">{dueCount} fällig</span>
    {/if}

    <div class="ml-auto flex gap-2">
      <button
        onclick={() => (showStats = true)}
        class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
        title="Statistiken"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zm6-4a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zm6-3a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
        </svg>
      </button>
      {#if cards.length > 0}
        <button
          onclick={onStudy}
          class="rounded-button bg-accent-correct text-white px-5 py-2 text-sm font-semibold hover:scale-[1.02] transition-transform"
        >
          Lernen
        </button>
      {/if}
      <button
        onclick={() => {
          showNewCard = true;
          editingCard = null;
          front = "";
          back = "";
          error = null;
        }}
        class="rounded-button bg-primary dark:bg-[#E0E0E0] dark:text-[#1A1A2E] text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
      >
        + Neue Karte
      </button>
    </div>
  </div>

  <!-- Error -->
  {#if error}
    <ErrorBanner message={error} onDismiss={() => (error = null)} />
  {/if}

  <!-- Card Form (New or Edit) -->
  {#if showNewCard || editingCard}
    <div class="px-6 pb-4">
      <div class="glass rounded-card p-4 space-y-3">
        <h3 class="text-sm font-medium text-secondary flex items-center gap-2">
          {editingCard ? "Karte bearbeiten" : "Neue Karteikarte"}
          <span
            class="inline-flex items-center justify-center w-4 h-4 rounded-full text-[10px] font-bold bg-secondary/20 text-secondary cursor-help"
            title="LaTeX wird unterstützt: $Formel$ für inline, $$Formel$$ für zentrierte Anzeige"
          >?</span>
        </h3>
        <div>
          <label class="text-xs font-medium text-secondary uppercase tracking-wide">Vorderseite</label>
          <textarea
            bind:value={front}
            placeholder="Frage eingeben..."
            class="w-full mt-1 bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-accent-correct transition-colors text-lg font-card"
            rows="2"
            autofocus
          ></textarea>
          {#if hasMath(front)}
            <div class="mt-2 p-3 rounded-lg border border-dashed border-accent-correct/40 bg-white/50 dark:bg-black/20">
              <span class="text-xs text-secondary mb-1 block">Vorschau</span>
              <div class="font-card text-primary dark:text-primary-dark text-sm">
                {@html renderMath(front)}
              </div>
            </div>
          {/if}
        </div>
        <div>
          <label class="text-xs font-medium text-secondary uppercase tracking-wide">Rückseite</label>
          <textarea
            bind:value={back}
            placeholder="Antwort eingeben..."
            class="w-full mt-1 bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-accent-correct transition-colors text-lg font-card"
            rows="2"
          ></textarea>
          {#if hasMath(back)}
            <div class="mt-2 p-3 rounded-lg border border-dashed border-accent-correct/40 bg-white/50 dark:bg-black/20">
              <span class="text-xs text-secondary mb-1 block">Vorschau</span>
              <div class="font-card text-primary dark:text-primary-dark text-sm">
                {@html renderMath(back)}
              </div>
            </div>
          {/if}
        </div>
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
              class="rounded-button bg-accent-correct text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
            >
              Speichern
            </button>
          {:else}
            <button
              onclick={handleCreate}
              disabled={!front.trim() || !back.trim()}
              class="rounded-button bg-accent-correct text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
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
        description="Erstelle deine erste Karteikarte in diesem Stapel. Mit Strg+Enter kannst du schnell erstellen."
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
          <div class="glass rounded-card p-4 flex items-start gap-4 group">
            <div class="flex-1 min-w-0 grid grid-cols-2 gap-4">
              <div>
                <span class="text-xs font-medium text-secondary uppercase tracking-wide">Frage</span>
                <p class="font-card text-primary dark:text-primary-dark mt-0.5 max-h-20 overflow-hidden">{@html renderMath(card.front)}</p>
              </div>
              <div>
                <span class="text-xs font-medium text-secondary uppercase tracking-wide">Antwort</span>
                <p class="font-card text-primary dark:text-primary-dark mt-0.5 max-h-20 overflow-hidden">{@html renderMath(card.back)}</p>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button
                class="p-1.5 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
                title="SM-2 Kartenzustand"
                onclick={() => toggleCardState(card)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
                </svg>
              </button>
              <button
                class="p-1.5 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
                title="Karte bearbeiten"
                onclick={() => startEdit(card)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                </svg>
              </button>
              <button
                class="p-1.5 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-accent-incorrect transition-colors"
                title="Karte löschen"
                onclick={() => handleDelete(card.id)}
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
          {#if cardStateOverlay?.card.id === card.id}
            <div class="glass rounded-lg p-3 mt-2 grid grid-cols-3 gap-2 text-xs">
              <div>
                <span class="text-secondary">Nächstes Review</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.next_review}</p>
              </div>
              <div>
                <span class="text-secondary">Intervall</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.interval} Tage</p>
              </div>
              <div>
                <span class="text-secondary">Ease-Faktor</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.ease_factor.toFixed(2)}</p>
              </div>
              <div>
                <span class="text-secondary">Wiederholungen</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.repetitions}</p>
              </div>
              <div>
                <span class="text-secondary">Korrekte Serie</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.correct_streak}</p>
              </div>
              <div>
                <span class="text-secondary">Reviews gesamt</span>
                <p class="text-primary dark:text-primary-dark font-medium">{cardStateOverlay.state.total_reviews}</p>
              </div>
            </div>
          {/if}
        {/each}
      </div>
    </div>
  {/if}
</div>
{/if}
