<script lang="ts">
  import { Archive, BookOpen, Play, RotateCcw } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { languageLabel } from "$lib/languages";
  import { renderMarkdown } from "$lib/markdown";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
  import type { Card, Deck, DeckStats } from "$lib/types";

  let {
    deck,
    refreshToken = 0,
    onSelect = (_deck: Deck) => {},
    onStudy = (_deck: Deck) => {},
    onPractice = (_deck: Deck) => {},
    onArchive = (_deck: Deck) => {},
  } = $props<{
    deck: Deck;
    refreshToken?: number;
    onSelect?: (deck: Deck) => void;
    onStudy?: (deck: Deck) => void;
    onPractice?: (deck: Deck) => void;
    onArchive?: (deck: Deck) => void;
  }>();

  let stats = $state<DeckStats | null>(null);
  let cards = $state<Card[]>([]);
  let loadedKey = $state("");
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  $effect(() => {
    const requestKey = `${deck.id}:${refreshToken}:${settingsStore.current.show_deck_card_previews}`;
    if (requestKey === loadedKey) return;
    loadedKey = requestKey;
    api.getDeckStats(deck.id).then((next) => {
      if (loadedKey === requestKey) stats = next;
    }).catch(() => {
      if (loadedKey === requestKey) stats = null;
    });
    if (!settingsStore.current.show_deck_card_previews) {
      cards = [];
      return;
    }
    api.listCards(deck.id).then((next) => {
      if (loadedKey === requestKey) cards = next;
    }).catch(() => {
      if (loadedKey === requestKey) cards = [];
    });
  });

  let cardCount = $derived(stats?.total_cards ?? 0);
  let dueCount = $derived(stats?.due_cards ?? 0);
  let previewCards = $derived(cards.slice(0, 3));
  const stackOffset = 4;
  let stackLayers = $derived.by(() => {
    if (!settingsStore.current.show_deck_card_previews || cardCount === 0) return 1;
    const countBasedLayers = Math.min(7, Math.ceil(cardCount / 10) + 1);
    return Math.max(countBasedLayers, previewCards.length + 1);
  });
</script>

<div class="relative w-full" style="aspect-ratio: 5 / 3" role="group" aria-label={t("Stapel")}>
  {#each Array(stackLayers - 1) as _, index}
    {@const card = previewCards[index]}
    {@const collapsedOffset = (index + 1) * stackOffset}
    <div
      class="glass-card pointer-events-none absolute left-0 top-0 h-[calc(100%-1.5rem)] w-[calc(100%-1.5rem)] overflow-hidden rounded-card border border-current/10 p-4 shadow-sm"
      style={`z-index: ${stackLayers - index}; transform: translate(${collapsedOffset}px, ${collapsedOffset}px);`}
      aria-hidden="true"
    >
      {#if card}
        <div class="flex h-full flex-col justify-between">
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2">
              <span class="section-kicker">{t("Frage")}</span>
              {#if card.front_language}
                <span class="rounded border border-accent-correct/30 bg-accent-correct/10 px-1.5 py-0.5 text-[10px] font-medium text-accent-correct">{languageLabel(card.front_language)}</span>
              {/if}
            </div>
            {#if card.tags.length > 0}
              <span class="max-w-24 truncate text-[10px] text-secondary">#{card.tags[0]}</span>
            {/if}
          </div>
          <div data-user-content class="{cardFontClass} line-clamp-3 text-center text-sm text-primary dark:text-primary-dark">
            {@html renderMarkdown(card.front)}
          </div>
        </div>
      {/if}
      <span class="absolute bottom-2 left-5 right-5 h-px bg-current/15"></span>
      <span class="absolute bottom-4 left-5 h-px w-2/5 bg-current/5"></span>
    </div>
  {/each}

  <article class="surface-panel relative z-10 flex h-[calc(100%-1.5rem)] w-[calc(100%-1.5rem)] flex-col overflow-hidden p-4 transition-colors hover:border-accent-correct/45">
    <header class="flex items-start justify-between gap-3">
      <button class="min-w-0 text-left" onclick={() => onSelect(deck)} title={t("Stapel öffnen")}>
        <p class="section-kicker mb-2">{t("Stapel")}</p>
        <h2 data-user-content class="{cardFontClass} line-clamp-2 text-xl leading-tight text-primary dark:text-primary-dark">{deck.name}</h2>
      </button>
      <button
        class="icon-button shrink-0 !h-9 !w-9"
        onclick={() => onArchive(deck)}
        title={t("Stapel archivieren")}
        aria-label={t("Stapel archivieren")}
      ><Archive size={17} /></button>
    </header>

    <footer class="mt-auto grid grid-cols-2 gap-2 border-t border-current/10 pt-4">
      <button class="secondary-action flex items-center justify-center gap-2 px-3 py-2 text-xs" onclick={() => onSelect(deck)}>
        <BookOpen size={15} /> {t("Öffnen")}
      </button>
      <button
        class="primary-action flex items-center justify-center gap-2 px-3 py-2 text-xs disabled:cursor-not-allowed disabled:opacity-45"
        onclick={() => onStudy(deck)}
        disabled={dueCount === 0}
      ><Play size={15} /> {t("Lernen")}</button>
      <button
        class="secondary-action col-span-2 flex items-center justify-center gap-2 px-3 py-2 text-xs disabled:cursor-not-allowed disabled:opacity-45"
        onclick={() => onPractice(deck)}
        disabled={cardCount === 0}
      ><RotateCcw size={15} /> {t("Freie Übung")}</button>
    </footer>
  </article>
</div>
