<script lang="ts">
  import * as api from "$lib/api";
  import type { SearchResult } from "$lib/types";
  import EmptyState from "./EmptyState.svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { fade } from "svelte/transition";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";

  let { initialQuery = "", onClose = () => {}, onSelectCard = (_deckId: string) => {} } = $props<{
    initialQuery?: string;
    onClose?: () => void;
    onSelectCard?: (deckId: string) => void;
  }>();

  let query = $state("");
  let results = $state<SearchResult[]>([]);
  let searched = $state(false);
  let loading = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);
  let initialSearchStarted = false;
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  $effect(() => {
    settingsStore.load();
    inputEl?.focus();
    if (initialQuery.trim() && !initialSearchStarted) {
      initialSearchStarted = true;
      query = initialQuery;
      void doSearch();
    }
  });

  async function doSearch() {
    if (!query.trim()) return;
    loading = true;
    searched = true;
    try {
      results = await api.searchCards(query.trim());
    } catch {
      results = [];
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
    if (e.key === "Enter") {
      doSearch();
    }
  }

  function handleCardClick(result: SearchResult) {
    onSelectCard(result.card.deck_id);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full overflow-hidden">
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
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark">
      Suche
    </h1>
  </div>

  <!-- Search bar -->
  <div class="px-6 pb-4">
    <div class="flex gap-2">
      <input
        bind:value={query}
        bind:this={inputEl}
        type="text"
        placeholder="Karten durchsuchen..."
        class="flex-1 bg-transparent border border-white/20 rounded-lg px-4 py-2 text-primary dark:text-primary-dark placeholder:text-secondary outline-none focus:border-accent-correct transition-colors"
        onkeydown={(e) => { if (e.key === "Enter") doSearch(); }}
      />
      <button
        onclick={doSearch}
        disabled={!query.trim()}
        class="rounded-button bg-accent-correct text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform disabled:opacity-50"
      >
        {t("Suchen")}
      </button>
    </div>
  </div>

  <!-- Results -->
  <div class="flex-1 grid overflow-hidden">
    {#if loading}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex-1 flex items-center justify-center">
      <p class="text-secondary">{t("Suche...")}</p>
    </div>
    {:else if searched && results.length === 0}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex-1 flex items-center justify-center">
      <EmptyState
        title="Keine Ergebnisse"
        description="Keine Karten gefunden, die deiner Suche entsprechen."
        icon={() => "🔍"}
      />
    </div>
    {:else if results.length > 0}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex-1 overflow-y-auto px-6 pb-6">
      <p class="text-secondary text-sm mb-3">{results.length} {t("Treffer")}</p>
      <div class="space-y-2">
        {#each results as result (result.card.id)}
          <button
            onclick={() => handleCardClick(result)}
            class="w-full glass rounded-card p-4 flex items-start gap-4 text-left hover:bg-white/5 transition-colors"
          >
            <div class="flex-1 min-w-0 flex flex-col gap-2">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <span class="text-xs font-medium text-secondary uppercase tracking-wide">Frage</span>
                  <div class="{cardFontClass} text-primary dark:text-primary-dark mt-0.5 line-clamp-2">{@html renderMarkdown(result.card.front)}</div>
                </div>
                <div>
                  <span class="text-xs font-medium text-secondary uppercase tracking-wide">Antwort</span>
                  <div class="{cardFontClass} text-primary dark:text-primary-dark mt-0.5 line-clamp-2">{@html renderMarkdown(result.card.back)}</div>
                </div>
              </div>
              {#if result.card.tags && result.card.tags.length > 0}
                <div class="flex flex-wrap gap-1 mt-1">
                  {#each result.card.tags as tag}
                    <span class="inline-flex items-center bg-white/10 text-secondary px-1.5 py-0.5 rounded text-[10px] font-medium">#{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
            <span class="text-xs text-secondary shrink-0 mt-1 bg-white/10 rounded-full px-2 py-0.5">
              {result.deck_name}
            </span>
          </button>
        {/each}
      </div>
      </div>
    {/if}
  </div>
</div>
