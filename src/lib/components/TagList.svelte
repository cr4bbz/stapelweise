<script lang="ts">
  import { ChevronDown, ChevronUp, Search } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { scale } from "svelte/transition";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import type { Card, Deck } from "$lib/types";

  let {
    decks = [],
    cards = [],
    onStudyTags = (_tags: string[]) => {},
  } = $props<{
    decks?: Deck[];
    cards?: Card[];
    onStudyTags: (tags: string[]) => void;
  }>();

  let tags = $state<string[]>([]);
  let selectedTags = $state<Set<string>>(new Set());
  let tagQuery = $state("");
  let selectedDeckIds = $state<string[]>([]);
  let pendingDeckIds = $state<string[]>([]);
  let deckFilterQuery = $state("");
  let showDeckFilter = $state(false);
  let tagsExpanded = $state(false);
  const collapsedTagRenderLimit = 48;
  const expansionHintLimit = 12;
  let deckTagNames = $derived.by(() => {
    if (!selectedDeckIds.length) return null;
    return new Set(cards.filter((card: Card) => selectedDeckIds.includes(card.deck_id)).flatMap((card: Card) => card.tags));
  });
  let matchingDecks = $derived(decks.filter((deck: Deck) => deck.name.toLocaleLowerCase().includes(deckFilterQuery.trim().toLocaleLowerCase())));
  let filteredTags = $derived(tags.filter((tag) => {
    const matchesDeck = !deckTagNames || deckTagNames.has(tag);
    return matchesDeck && tag.toLocaleLowerCase().includes(tagQuery.trim().toLocaleLowerCase());
  }));
  let visibleTags = $derived(tagsExpanded || tagQuery.trim() ? filteredTags : filteredTags.slice(0, collapsedTagRenderLimit));
  let canExpandTags = $derived(filteredTags.length > expansionHintLimit);

  onMount(async () => {
    try {
      tags = await api.getAllTags();
    } catch {
      tags = [];
    }
  });

  function toggleTag(tag: string) {
    const next = new Set(selectedTags);
    if (next.has(tag)) {
      next.delete(tag);
    } else {
      next.add(tag);
    }
    selectedTags = next;
  }

  function openDeckFilter() {
    pendingDeckIds = [...selectedDeckIds];
    deckFilterQuery = "";
    showDeckFilter = true;
  }

  function togglePendingDeck(deckId: string) {
    pendingDeckIds = pendingDeckIds.includes(deckId)
      ? pendingDeckIds.filter((selectedId) => selectedId !== deckId)
      : [...pendingDeckIds, deckId];
  }

  function applyDeckFilter() {
    selectedDeckIds = [...pendingDeckIds];
    selectedTags = new Set();
    tagsExpanded = false;
    showDeckFilter = false;
    deckFilterQuery = "";
  }
</script>

<div class="h-full min-h-0">
  <div class="surface-panel tag-list-panel flex h-full min-h-0 flex-col overflow-visible p-4">
    <div class="mb-3 flex items-center justify-between gap-3">
      <span class="section-kicker">Tags ({filteredTags.length})</span>
    </div>

    <div class="mb-3 grid grid-cols-[minmax(0,1fr)_auto] gap-2">
      <label class="relative min-w-0">
        <Search class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-secondary" size={14} aria-hidden="true" />
        <input
          bind:value={tagQuery}
          type="search"
          class="module-accent-input w-full rounded-md py-2 pl-8 pr-8 text-xs outline-none"
          placeholder={t("Tags durchsuchen")}
          aria-label={t("Tags durchsuchen")}
        />
      </label>
      <div class="relative min-w-0">
        <button
          type="button"
          onclick={openDeckFilter}
          class="module-accent-input flex h-full max-w-[9rem] items-center gap-1 rounded-md px-2 py-2 text-xs font-semibold outline-none"
          aria-expanded={showDeckFilter}
          aria-haspopup="dialog"
          aria-label={t("Nach Stapel filtern")}
        >
          <span class="min-w-0 truncate">{selectedDeckIds.length ? `${selectedDeckIds.length} ${t("Stapel")}` : t("Alle Stapel")}</span>
          <ChevronDown class="shrink-0" size={14} aria-hidden="true" />
        </button>
        {#if showDeckFilter}
          <div in:scale={{ duration: 160, start: 0.96 }} out:scale={{ duration: 120, start: 0.96 }} class="tag-deck-filter-popover glass absolute right-0 top-full z-30 mt-2 w-64 origin-top-right rounded-card border border-white/20 p-2 shadow-elevation-high" role="dialog" aria-label={t("Nach Stapel filtern")}>
            <input
              bind:value={deckFilterQuery}
              class="module-accent-input w-full rounded-md px-3 py-2 text-sm outline-none"
              placeholder={t("Stapel suchen...")}
              aria-label={t("Stapel suchen...")}
            />
            <div class="mt-2 max-h-40 space-y-1 overflow-y-auto">
              <button
                type="button"
                onclick={() => (pendingDeckIds = [])}
                class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs font-semibold transition-colors {pendingDeckIds.length === 0 ? 'module-accent-selected' : 'module-accent-soft text-primary dark:text-primary-dark'}"
              >
                <span class="h-3 w-3 rounded-sm border border-current/40 {pendingDeckIds.length === 0 ? 'bg-white/70' : ''}"></span>
                {t("Alle Stapel")}
              </button>
              {#each matchingDecks as deck}
                <button
                  type="button"
                  onclick={() => togglePendingDeck(deck.id)}
                  class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs font-semibold transition-colors {pendingDeckIds.includes(deck.id) ? 'module-accent-selected' : 'module-accent-soft text-primary dark:text-primary-dark'}"
                  aria-pressed={pendingDeckIds.includes(deck.id)}
                >
                  <span class="h-3 w-3 rounded-sm border border-current/40 {pendingDeckIds.includes(deck.id) ? 'bg-white/70' : ''}"></span>
                  <span class="truncate">{deck.name}</span>
                </button>
              {/each}
            </div>
            <div class="mt-2 flex justify-end gap-2 border-t border-current/10 pt-2">
              <button type="button" onclick={() => (showDeckFilter = false)} class="secondary-action px-2.5 py-1.5 text-xs">{t("Abbrechen")}</button>
              <button type="button" onclick={applyDeckFilter} class="primary-action px-2.5 py-1.5 text-xs">{t("Anwenden")}</button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    {#if visibleTags.length > 0}
      <div class="custom-scrollbar tag-list-items flex flex-wrap content-start gap-2 pr-2" class:tag-list-items-expanded={tagsExpanded || Boolean(tagQuery.trim())}>
        {#each visibleTags as tag}
          <button
            onclick={() => toggleTag(tag)}
            class="inline-flex cursor-pointer items-center rounded-lg border px-3 py-1 text-xs font-medium transition-colors {selectedTags.has(tag)
              ? 'module-accent-selected shadow-sm'
              : 'module-accent-soft text-primary dark:text-primary-dark'}"
          >
            #{tag}
          </button>
        {/each}
      </div>
    {:else}
      <div class="flex min-h-0 flex-1 items-center justify-center text-center">
        <p class="text-sm text-secondary">
          {tagQuery.trim() || selectedDeckIds.length ? t("Keine passenden Tags.") : t("Tags erscheinen hier, sobald du sie einer Karte zuweist.")}
        </p>
      </div>
    {/if}

    {#if tags.length > 0}
      <footer class="module-list-footer mt-auto min-h-11 border-t border-current/10 pt-3">
        {#if selectedTags.size > 0}
          <button
            onclick={() => {
              onStudyTags(Array.from(selectedTags));
              selectedTags = new Set();
            }}
            class="primary-action flex h-8 w-full items-center justify-center !rounded-md px-3 py-0 text-xs"
          >
            {selectedTags.size} Tags lernen
          </button>
        {:else if canExpandTags && !tagQuery.trim()}
          <button
            type="button"
            class="secondary-action flex h-8 w-full items-center justify-center gap-1.5 !rounded-md px-3 py-0 text-xs"
            onclick={() => (tagsExpanded = !tagsExpanded)}
            aria-expanded={tagsExpanded}
          >
            {#if tagsExpanded}
              <ChevronUp size={14} aria-hidden="true" />
              {t("Weniger Tags anzeigen")}
            {:else}
              <ChevronDown size={14} aria-hidden="true" />
              {t("Mehr Tags anzeigen")}
            {/if}
          </button>
        {:else}
          <div class="h-8" aria-hidden="true"></div>
        {/if}
      </footer>
    {/if}
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }

  .custom-scrollbar {
    scrollbar-width: thin;
    scrollbar-color: rgba(150, 150, 150, 0.5) transparent;
  }

  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }

  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(150, 150, 150, 0.3);
    border-radius: 10px;
  }

  .tag-list-items {
    flex: 1 1 auto;
    min-height: 0;
    overflow: hidden;
    scrollbar-gutter: stable;
  }

  .tag-list-items-expanded {
    overflow-y: auto;
  }
</style>
