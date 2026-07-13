<script lang="ts">
  import * as api from "$lib/api";
  import type { Card, CardState, CardType, Deck } from "$lib/types";
  import EmptyState from "./EmptyState.svelte";
  import ErrorBanner from "./ErrorBanner.svelte";
  import StatsDashboard from "./StatsDashboard.svelte";
  import BrowseCards from "./BrowseCards.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import FlashCard from "./FlashCard.svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { hasMath } from "$lib/math";
  import { mediaStore } from "$lib/stores/media";
  import { tick } from "svelte";
  import { fade, slide } from "svelte/transition";

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
  let reasoning = $state("");
  let editingCard = $state<Card | null>(null);
  let error = $state<string | null>(null);
  let dueCount = $state<number | null>(null);
  let showStats = $state(false);
  let showBrowse = $state(false);
  let deleteConfirmCardId = $state<string | null>(null);
  let viewingCard = $state<Card | null>(null);
  let cardFlipped = $state(false);
  let frontTextarea = $state<HTMLTextAreaElement | null>(null);

  let tags = $state<string[]>([]);
  let tagInput = $state("");
  let availableTags = $state<string[]>([]);
  let selectedFilterTag = $state<string | null>(null);

  let matchingTags = $derived(
    tagInput.trim()
      ? availableTags.filter((t) => t.toLowerCase().includes(tagInput.trim().toLowerCase()) && !tags.includes(t))
      : []
  );

  let allDeckTags = $derived(
    (Array.from(new Set(cards.flatMap((c) => c.tags || []))) as string[]).sort()
  );

  let filteredCards = $derived(
    selectedFilterTag ? cards.filter((c) => c.tags?.includes(selectedFilterTag!)) : cards
  );

  async function loadTags() {
    try {
      availableTags = await api.getAllTags();
    } catch {
      // ignore
    }
  }

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
    loadTags();
  });

  $effect(() => {
    if (showNewCard || editingCard) {
      tick().then(() => frontTextarea?.focus());
    }
  });

  function addTag(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      const newTag = tagInput.trim().toLowerCase();
      if (newTag && !tags.includes(newTag)) {
        tags = [...tags, newTag];
      }
      tagInput = "";
    }
  }

  function removeTag(tag: string) {
    tags = tags.filter(t => t !== tag);
  }

  function processImageFile(file: File, targetField: 'front' | 'back' | 'reasoning', textareaEl?: HTMLTextAreaElement | null) {
    if (file.size > 5 * 1024 * 1024) {
      error = "Das gewählte Bild ist zu groß (maximal 5 MB erlaubt).";
      return;
    }
    const reader = new FileReader();
    reader.onload = (re) => {
      const result = re.target?.result as string;
      if (!result) return;
      const mediaRef = mediaStore.saveMedia(result);
      const imgTag = `\n![${file.name || 'Bild'}](${mediaRef})\n`;

      if (targetField === "front") {
        if (textareaEl) {
          const start = textareaEl.selectionStart || front.length;
          const end = textareaEl.selectionEnd || front.length;
          front = front.slice(0, start) + imgTag + front.slice(end);
        } else {
          front += imgTag;
        }
      } else if (targetField === "back") {
        if (textareaEl) {
          const start = textareaEl.selectionStart || back.length;
          const end = textareaEl.selectionEnd || back.length;
          back = back.slice(0, start) + imgTag + back.slice(end);
        } else {
          back += imgTag;
        }
      } else if (targetField === "reasoning") {
        if (textareaEl) {
          const start = textareaEl.selectionStart || reasoning.length;
          const end = textareaEl.selectionEnd || reasoning.length;
          reasoning = reasoning.slice(0, start) + imgTag + reasoning.slice(end);
        } else {
          reasoning += imgTag;
        }
      }
    };
    reader.readAsDataURL(file);
  }

  function handleImageUpload(targetField: 'front' | 'back' | 'reasoning', textareaEl?: HTMLTextAreaElement | null) {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.onchange = (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      processImageFile(file, targetField, textareaEl);
    };
    input.click();
  }

  function handlePaste(e: ClipboardEvent, targetField: 'front' | 'back' | 'reasoning') {
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (item.type.indexOf("image") !== -1) {
        const file = item.getAsFile();
        if (file) {
          e.preventDefault();
          processImageFile(file, targetField, e.target as HTMLTextAreaElement);
          break;
        }
      }
    }
  }

  function handleDrop(e: DragEvent, targetField: 'front' | 'back' | 'reasoning') {
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        if (file.type.startsWith("image/")) {
          e.preventDefault();
          processImageFile(file, targetField, e.target as HTMLTextAreaElement);
          break;
        }
      }
    }
  }

  let cardType = $state<CardType>("basic");
  let mcOptions = $state<{ text: string; correct: boolean }[]>([
    { text: "", correct: true },
    { text: "", correct: false },
  ]);
  let orderingItems = $state<string[]>(["", ""]);

  function addMcOption() {
    mcOptions = [...mcOptions, { text: "", correct: false }];
  }

  function removeMcOption(idx: number) {
    if (mcOptions.length > 2) {
      mcOptions = mcOptions.filter((_, i) => i !== idx);
    }
  }

  function addOrderingItem() {
    orderingItems = [...orderingItems, ""];
  }

  function removeOrderingItem(idx: number) {
    if (orderingItems.length > 2) {
      orderingItems = orderingItems.filter((_, i) => i !== idx);
    }
  }

  function startEdit(card: Card) {
    editingCard = card;
    front = mediaStore.compactMarkdown(card.front);
    back = mediaStore.compactMarkdown(card.back);
    reasoning = mediaStore.compactMarkdown(card.reasoning || "");
    tags = [...card.tags];
    cardType = card.card_type || "basic";

    if (cardType === "multiple_choice") {
      try {
        if (card.content) {
          const parsed = JSON.parse(card.content);
          if (Array.isArray(parsed?.options)) mcOptions = parsed.options;
        }
      } catch {
        // fallback
      }
    } else if (cardType === "ordering") {
      try {
        if (card.content) {
          const parsed = JSON.parse(card.content);
          if (Array.isArray(parsed?.items)) orderingItems = parsed.items;
        }
      } catch {
        // fallback
      }
    }
    showNewCard = false;
  }

  function cancelEdit() {
    editingCard = null;
    showNewCard = false;
    front = "";
    back = "";
    reasoning = "";
    tags = [];
    tagInput = "";
    cardType = "basic";
    mcOptions = [
      { text: "", correct: true },
      { text: "", correct: false },
    ];
    orderingItems = ["", ""];
  }

  async function handleCreate() {
    if (!front.trim()) return;
    
    let cType: CardType = cardType;
    let finalBack = back.trim();
    let contentJson: string | null = null;

    if (cType === "multiple_choice") {
      const validMc = mcOptions.filter(o => o.text.trim());
      if (validMc.length < 2 || !validMc.some(o => o.correct)) {
        error = "Eine Multiple-Choice-Karte benötigt mindestens 2 Optionen und mindestens eine richtige Antwort.";
        return;
      }
      contentJson = JSON.stringify({ options: validMc });
      finalBack = validMc.map(o => `[${o.correct ? "x" : " "}] ${o.text}`).join("\n");
    } else if (cType === "ordering") {
      const validOrd = orderingItems.filter(i => i.trim());
      if (validOrd.length < 2) {
        error = "Eine Reihenfolge-Karte benötigt mindestens 2 Schritte.";
        return;
      }
      contentJson = JSON.stringify({ items: validOrd });
      finalBack = validOrd.map((item, idx) => `${idx + 1}. ${item}`).join("\n");
    } else {
      if (!finalBack) return;
      if (front.includes("==") || front.includes("{{c1::")) {
        cType = "cloze";
      }
    }

    error = null;
    try {
      const card = await api.createCard(deck.id, front.trim(), finalBack, reasoning.trim() || null, cType, contentJson, tags);
      cards = [card, ...cards];
      cancelEdit();
      loadDueCount();
      loadTags();
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Erstellen der Karte";
    }
  }

  async function handleUpdate() {
    const card = editingCard;
    if (!card || !front.trim()) return;
    
    let cType: CardType = cardType;
    let finalBack = back.trim();
    let contentJson: string | null = null;

    if (cType === "multiple_choice") {
      const validMc = mcOptions.filter(o => o.text.trim());
      if (validMc.length < 2 || !validMc.some(o => o.correct)) {
        error = "Eine Multiple-Choice-Karte benötigt mindestens 2 Optionen und mindestens eine richtige Antwort.";
        return;
      }
      contentJson = JSON.stringify({ options: validMc });
      finalBack = validMc.map(o => `[${o.correct ? "x" : " "}] ${o.text}`).join("\n");
    } else if (cType === "ordering") {
      const validOrd = orderingItems.filter(i => i.trim());
      if (validOrd.length < 2) {
        error = "Eine Reihenfolge-Karte benötigt mindestens 2 Schritte.";
        return;
      }
      contentJson = JSON.stringify({ items: validOrd });
      finalBack = validOrd.map((item, idx) => `${idx + 1}. ${item}`).join("\n");
    } else {
      if (!finalBack) return;
      if (front.includes("==") || front.includes("{{c1::")) {
        cType = "cloze";
      }
    }

    error = null;
    try {
      await api.updateCard(card.id, front.trim(), finalBack, reasoning.trim() || null, cType, contentJson, tags);
      cards = cards.map((c) =>
        c.id === card.id ? ({ ...c, card_type: cType, content: contentJson, front: front.trim(), back: finalBack, reasoning: reasoning.trim() || null, tags } as Card) : c
      );
      cancelEdit();
      loadTags();
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
      loadTags();
    } catch (e: any) {
      error = e?.toString() || "Fehler beim Löschen der Karte";
    }
  }



  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (viewingCard) {
        viewingCard = null;
        cardFlipped = false;
        return;
      }
      if (editingCard || showNewCard) {
        cancelEdit();
      } else {
        onClose();
      }
    }
    if (viewingCard) {
      if (e.key === "Enter" || e.key === " " || e.key === "Spacebar") {
        e.preventDefault();
        cardFlipped = !cardFlipped;
      }
      return;
    }
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      if (editingCard) handleUpdate();
      else if (showNewCard) handleCreate();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex-1 grid overflow-y-auto w-full h-full">
  {#if showStats}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full">
    <StatsDashboard deckId={deck.id} deckName={deck.name} onClose={() => (showStats = false)} />
    </div>
  {:else if showBrowse}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 h-full w-full">
    <BrowseCards cards={cards} deckName={deck.name} onClose={() => (showBrowse = false)} />
    </div>
  {:else if viewingCard}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex flex-col h-full w-full">
    <div class="flex items-center gap-3 p-6 pb-2">
      <button
        onclick={() => { viewingCard = null; cardFlipped = false; }}
        class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
        title="Zurück (Esc)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
        </svg>
      </button>
      <h1 class="text-xl font-bold text-primary dark:text-primary-dark truncate">
        {deck.name}
      </h1>
      <span class="text-secondary text-sm ml-auto">Klick zum Umdrehen</span>
    </div>
    <div class="flex-1 flex flex-col items-center justify-center px-6 pb-6 gap-6">
      <div
        class="flex flex-col items-center gap-6 w-full cursor-pointer"
        role="button"
        tabindex="0"
        onclick={() => (cardFlipped = !cardFlipped)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " " || e.key === "Spacebar") {
            e.preventDefault();
            cardFlipped = !cardFlipped;
          }
        }}
      >
        <FlashCard
          front={viewingCard.front}
          back={viewingCard.back}
          reasoning={viewingCard.reasoning}
          tags={viewingCard.tags}
          flipped={cardFlipped}
          cardType={viewingCard.card_type}
          content={viewingCard.content}
        />
      </div>
      <button
        onclick={() => (cardFlipped = !cardFlipped)}
        class="rounded-button bg-accent-correct text-white px-5 py-2 text-sm font-semibold hover:scale-[1.02] transition-transform"
      >
        {cardFlipped ? "Vorderseite zeigen" : "Rückseite zeigen"}
      </button>
      </div>
    </div>
  {:else}
    <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex flex-col h-full w-full">
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
          onclick={() => (showBrowse = true)}
          class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
          title="Durchblättern"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z" />
          </svg>
        </button>
      {/if}
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
          reasoning = "";
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
    <div transition:slide={{ duration: 200, axis: "y" }} class="px-6 pb-4">
      <div class="glass rounded-card p-4 space-y-3">
        <div class="flex items-center justify-between border-b border-white/10 pb-3">
          <h3 class="text-sm font-medium text-secondary flex items-center gap-2">
            {editingCard ? "Karte bearbeiten" : "Neue Karteikarte"}
          </h3>
          <!-- Card Type Selector -->
          <div class="flex items-center gap-1.5 glass p-1 rounded-xl border border-white/10">
            <button
              type="button"
              onclick={() => cardType = "basic"}
              class="px-2.5 py-1 text-xs font-semibold rounded-lg transition-all {cardType === 'basic' ? 'bg-accent-correct text-white shadow-sm' : 'text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              📝 Standard
            </button>
            <button
              type="button"
              onclick={() => cardType = "multiple_choice"}
              class="px-2.5 py-1 text-xs font-semibold rounded-lg transition-all {cardType === 'multiple_choice' ? 'bg-accent-correct text-white shadow-sm' : 'text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              🔘 Multiple Choice
            </button>
            <button
              type="button"
              onclick={() => cardType = "ordering"}
              class="px-2.5 py-1 text-xs font-semibold rounded-lg transition-all {cardType === 'ordering' ? 'bg-accent-correct text-white shadow-sm' : 'text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              🔢 Reihenfolge
            </button>
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-secondary uppercase tracking-wide">
              {cardType === 'ordering' ? 'Titel / Fragestellung' : cardType === 'multiple_choice' ? 'Frage / Aufgabenstellung' : 'Vorderseite'}
            </span>
            <button
              type="button"
              onclick={(e) => handleImageUpload('front', (e.currentTarget.parentElement?.nextElementSibling as HTMLTextAreaElement))}
              class="text-xs font-medium text-secondary hover:text-accent-correct transition-colors flex items-center gap-1"
              title="Klicken zum Auswählen, oder Bild direkt im Textfeld einfügen / per Drag & Drop ablegen"
            >
              🖼️ Bild einfügen
            </button>
          </div>
          <textarea
            bind:this={frontTextarea}
            bind:value={front}
            onpaste={(e) => handlePaste(e, 'front')}
            ondrop={(e) => handleDrop(e, 'front')}
            ondragover={(e) => e.preventDefault()}
            placeholder={cardType === 'ordering' ? 'z. B. Bringe die Schritte in die richtige Reihenfolge:' : cardType === 'multiple_choice' ? 'z. B. Welche der folgenden Aussagen treffen zu?' : 'Frage eingeben (Strg+V oder Drag&Drop für Bilder)...'}
            class="w-full mt-1 bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-accent-correct transition-colors text-lg font-card"
            rows="2"
          ></textarea>
          {#if hasMath(front) || front.includes('![')}
            <div class="mt-2 p-3 rounded-lg border border-dashed border-accent-correct/40 bg-white/50 dark:bg-black/20">
              <span class="text-xs text-secondary mb-1 block">Live-Vorschau</span>
              <div class="font-card text-primary dark:text-primary-dark text-sm max-h-48 overflow-y-auto">
                {@html renderMarkdown(front)}
              </div>
            </div>
          {/if}
        </div>

        {#if cardType === 'multiple_choice'}
          <!-- Multiple Choice Options Editor -->
          <div class="space-y-2 pt-2 border-t border-white/10">
            <div class="text-xs font-medium text-secondary uppercase tracking-wide flex items-center justify-between">
              <span>Antwortoptionen</span>
              <span class="text-[10px] lowercase text-secondary/70">Checkmark [x] = Richtig</span>
            </div>
            <div class="space-y-2">
              {#each mcOptions as opt, idx}
                <div class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    bind:checked={opt.correct}
                    title="Als richtige Antwort markieren"
                    class="w-5 h-5 accent-accent-correct cursor-pointer"
                  />
                  <input
                    type="text"
                    bind:value={opt.text}
                    placeholder={`Option ${idx + 1}`}
                    class="flex-1 bg-white/5 dark:bg-black/20 border border-white/10 rounded-lg p-2 text-primary dark:text-primary-dark placeholder:text-secondary/50 outline-none focus:border-accent-correct text-sm font-card"
                  />
                  {#if mcOptions.length > 2}
                    <button
                      type="button"
                      onclick={() => removeMcOption(idx)}
                      class="p-1.5 rounded-lg text-secondary hover:text-red-400 transition-colors"
                      title="Option entfernen"
                    >
                      ✕
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
            <button
              type="button"
              onclick={addMcOption}
              class="mt-2 text-xs font-semibold text-accent-correct hover:underline flex items-center gap-1"
            >
              + Weitere Option hinzufügen
            </button>
          </div>

        {:else if cardType === 'ordering'}
          <!-- Ordering Steps Editor -->
          <div class="space-y-2 pt-2 border-t border-white/10">
            <span class="text-xs font-medium text-secondary uppercase tracking-wide block">
              Prozessschritte (in der KORREKTEN Reihenfolge eingeben)
            </span>
            <div class="space-y-2">
              {#each orderingItems as item, idx}
                <div class="flex items-center gap-2">
                  <span class="text-xs font-bold text-secondary w-5">{idx + 1}.</span>
                  <input
                    type="text"
                    bind:value={orderingItems[idx]}
                    placeholder={`Schritt ${idx + 1}`}
                    class="flex-1 bg-white/5 dark:bg-black/20 border border-white/10 rounded-lg p-2 text-primary dark:text-primary-dark placeholder:text-secondary/50 outline-none focus:border-accent-correct text-sm font-card"
                  />
                  {#if orderingItems.length > 2}
                    <button
                      type="button"
                      onclick={() => removeOrderingItem(idx)}
                      class="p-1.5 rounded-lg text-secondary hover:text-red-400 transition-colors"
                      title="Schritt entfernen"
                    >
                      ✕
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
            <button
              type="button"
              onclick={addOrderingItem}
              class="mt-2 text-xs font-semibold text-accent-correct hover:underline flex items-center gap-1"
            >
              + Weiteren Schritt hinzufügen
            </button>
          </div>

        {:else}
          <!-- Standard Backside Input -->
          <div>
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-secondary uppercase tracking-wide">Rückseite</span>
              <button
                type="button"
                onclick={(e) => handleImageUpload('back', (e.currentTarget.parentElement?.nextElementSibling as HTMLTextAreaElement))}
                class="text-xs font-medium text-secondary hover:text-accent-correct transition-colors flex items-center gap-1"
                title="Klicken zum Auswählen, oder Bild direkt im Textfeld einfügen / per Drag & Drop ablegen"
              >
                🖼️ Bild einfügen
              </button>
            </div>
            <textarea
              bind:value={back}
              onpaste={(e) => handlePaste(e, 'back')}
              ondrop={(e) => handleDrop(e, 'back')}
              ondragover={(e) => e.preventDefault()}
              placeholder="Antwort eingeben (Strg+V oder Drag&Drop für Bilder)..."
              class="w-full mt-1 bg-transparent border border-white/20 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary resize-none outline-none focus:border-accent-correct transition-colors text-lg font-card"
              rows="2"
            ></textarea>
            {#if hasMath(back) || back.includes('![')}
              <div class="mt-2 p-3 rounded-lg border border-dashed border-accent-correct/40 bg-white/50 dark:bg-black/20">
                <span class="text-xs text-secondary mb-1 block">Live-Vorschau</span>
                <div class="font-card text-primary dark:text-primary-dark text-sm max-h-48 overflow-y-auto">
                  {@html renderMarkdown(back)}
                </div>
              </div>
            {/if}
          </div>
        {/if}
        <div class="mt-4 pt-4 border-t border-white/10">
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs font-medium text-secondary uppercase tracking-wide flex items-center gap-2">
              Warum? <span class="text-[10px] lowercase opacity-70">(Optional, fördert Verstehen)</span>
            </span>
            <button
              type="button"
              onclick={(e) => handleImageUpload('reasoning', (e.currentTarget.parentElement?.nextElementSibling as HTMLTextAreaElement))}
              class="text-xs font-medium text-secondary hover:text-accent-correct transition-colors flex items-center gap-1"
              title="Klicken zum Auswählen, oder Bild direkt im Textfeld einfügen / per Drag & Drop ablegen"
            >
              🖼️ Bild einfügen
            </button>
          </div>
          <textarea
            bind:value={reasoning}
            onpaste={(e) => handlePaste(e, 'reasoning')}
            ondrop={(e) => handleDrop(e, 'reasoning')}
            ondragover={(e) => e.preventDefault()}
            placeholder="Warum ist diese Antwort richtig? Wie hängt sie mit anderem Wissen zusammen? (Strg+V oder Drag&Drop für Bilder)"
            class="w-full mt-1 bg-white/5 dark:bg-black/20 border border-white/10 rounded-lg p-3 text-primary dark:text-primary-dark placeholder:text-secondary/50 resize-none outline-none focus:border-accent-correct/50 transition-colors text-sm font-card"
            rows="2"
          ></textarea>
          {#if hasMath(reasoning) || reasoning.includes('![')}
            <div class="mt-2 p-3 rounded-lg border border-dashed border-accent-correct/40 bg-white/50 dark:bg-black/20">
              <span class="text-xs text-secondary mb-1 block">Live-Vorschau</span>
              <div class="font-card text-primary dark:text-primary-dark text-sm max-h-48 overflow-y-auto">
                {@html renderMarkdown(reasoning)}
              </div>
            </div>
          {/if}
        </div>
        <div class="mt-2 pt-4 border-t border-white/10">
          <span class="text-xs font-medium text-secondary uppercase tracking-wide flex items-center gap-2 mb-2">
            Tags <span class="text-[10px] lowercase opacity-70">(Mit Komma oder Enter trennen)</span>
          </span>
          <div class="flex flex-wrap gap-2 mb-2">
            {#each tags as tag}
              <span class="inline-flex items-center gap-1 bg-accent-correct/20 text-accent-correct px-2 py-1 rounded text-xs font-medium">
                #{tag}
                <button onclick={() => removeTag(tag)} class="hover:text-white transition-colors" type="button" title="Tag entfernen" aria-label="Tag #{tag} entfernen">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                  </svg>
                </button>
              </span>
            {/each}
          </div>
          <div class="relative">
            <input
              type="text"
              bind:value={tagInput}
              onkeydown={addTag}
              placeholder="Tags eingeben..."
              class="w-full bg-white/5 dark:bg-black/20 border border-white/10 rounded-lg p-2 text-primary dark:text-primary-dark placeholder:text-secondary/50 outline-none focus:border-accent-correct/50 transition-colors text-sm font-card"
            />
            {#if matchingTags.length > 0}
              <div class="absolute z-30 left-0 right-0 top-full mt-1 glass rounded-lg border border-white/20 shadow-elevation-high p-1.5 flex flex-wrap gap-1 max-h-32 overflow-y-auto">
                <span class="text-[10px] text-secondary w-full px-1">Vorschläge:</span>
                {#each matchingTags as match}
                  <button
                    type="button"
                    onclick={() => {
                      if (!tags.includes(match)) tags = [...tags, match];
                      tagInput = "";
                    }}
                    class="text-xs px-2 py-0.5 rounded bg-accent-correct/10 hover:bg-accent-correct/30 text-accent-correct font-medium transition-colors"
                  >
                    +#{match}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <div class="flex gap-2 justify-end mt-2">
          <button
            onclick={cancelEdit}
            class="text-secondary text-sm hover:text-primary dark:hover:text-primary-dark"
          >
            Abbrechen
          </button>
          {#if editingCard}
            <button
              onclick={handleUpdate}
              disabled={!front.trim() || (cardType === 'basic' && !back.trim())}
              class="rounded-button bg-accent-correct text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
            >
              Speichern
            </button>
          {:else}
            <button
              onclick={handleCreate}
              disabled={!front.trim() || (cardType === 'basic' && !back.trim())}
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
          reasoning = "";
          tags = [];
          tagInput = "";
        }}
        icon={() => "🃏"}
      />
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto min-h-0 px-6 pb-6">
      {#if allDeckTags.length > 0}
        <div class="flex flex-wrap items-center gap-1.5 mb-3">
          <span class="text-xs text-secondary font-medium mr-1">Filter nach Tag:</span>
          <button
            onclick={() => (selectedFilterTag = null)}
            class="text-xs px-2.5 py-1 rounded-lg font-medium transition-all {selectedFilterTag === null ? 'bg-accent-correct text-white shadow-sm' : 'glass text-secondary hover:text-primary dark:hover:text-primary-dark'}"
          >
            Alle ({cards.length})
          </button>
          {#each allDeckTags as tag}
            {@const count = cards.filter(c => c.tags?.includes(tag)).length}
            <button
              onclick={() => (selectedFilterTag = selectedFilterTag === tag ? null : tag)}
              class="text-xs px-2.5 py-1 rounded-lg font-medium transition-all flex items-center gap-1 {selectedFilterTag === tag ? 'bg-accent-correct text-white shadow-sm' : 'glass text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              <span>#{tag}</span>
              <span class="text-[10px] opacity-70">({count})</span>
            </button>
          {/each}
        </div>
      {/if}

      <div class="space-y-2">
        {#each filteredCards as card (card.id)}
          <div class="glass rounded-card p-4 flex items-start gap-4 group cursor-pointer hover:bg-white/5 dark:hover:bg-white/5 transition-colors" onclick={() => { viewingCard = card; cardFlipped = false; }} role="button" tabindex="0" onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (viewingCard = card, cardFlipped = false)}>
            <div class="flex-1 min-w-0 flex flex-col gap-2">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <div class="flex items-center gap-2 mb-0.5">
                    <span class="text-xs font-medium text-secondary uppercase tracking-wide">Frage</span>
                    {#if card.card_type === 'multiple_choice'}
                      <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 border border-blue-500/30">🔘 MC</span>
                    {:else if card.card_type === 'ordering'}
                      <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400 border border-purple-500/30">🔢 Sequenz</span>
                    {:else if card.card_type === 'cloze'}
                      <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-400 border border-amber-500/30">🧩 Cloze</span>
                    {/if}
                  </div>
                  <p class="font-card text-primary dark:text-primary-dark mt-0.5 max-h-20 overflow-hidden">{@html renderMarkdown(card.front)}</p>
                </div>
                <div>
                  <span class="text-xs font-medium text-secondary uppercase tracking-wide">Antwort</span>
                  <p class="font-card text-primary dark:text-primary-dark mt-0.5 max-h-20 overflow-hidden">{@html renderMarkdown(card.back)}</p>
                </div>
              </div>
              {#if card.tags && card.tags.length > 0}
                <div class="flex flex-wrap gap-1 mt-1">
                  {#each card.tags as tag}
                    <span class="inline-flex items-center bg-white/10 text-secondary px-1.5 py-0.5 rounded text-[10px] font-medium">#{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
            <div class="flex items-center gap-1 shrink-0" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="presentation">
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
                onclick={() => (deleteConfirmCardId = card.id)}
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
</div>

{#if deleteConfirmCardId}
  <ConfirmDialog
    message="Diese Karte wird dauerhaft gelöscht."
    confirmLabel="Löschen"
    onConfirm={() => {
      handleDelete(deleteConfirmCardId!);
      deleteConfirmCardId = null;
    }}
    onCancel={() => (deleteConfirmCardId = null)}
  />
{/if}
