<script lang="ts">
  import { Archive, BookOpen, Play, RotateCcw } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
  import type { Deck, DeckStats } from "$lib/types";

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
  let loadedKey = $state("");
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  $effect(() => {
    const requestKey = `${deck.id}:${refreshToken}`;
    if (requestKey === loadedKey) return;
    loadedKey = requestKey;
    api.getDeckStats(deck.id).then((next) => (stats = next)).catch(() => (stats = null));
  });

  let cardCount = $derived(stats?.total_cards ?? 0);
  let dueCount = $derived(stats?.due_cards ?? 0);
</script>

<article class="surface-panel flex h-full min-h-56 flex-col overflow-hidden p-4 transition-colors hover:border-accent-correct/45">
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

  <div class="mt-auto grid grid-cols-2 gap-3 pt-6">
    <div class="module-accent-subpanel rounded-lg p-3">
      <p class="text-xs text-secondary">{t("Karten")}</p>
      <p class="font-pixel mt-1 text-sm font-bold text-primary dark:text-primary-dark">{cardCount}</p>
    </div>
    <div class="module-accent-subpanel rounded-lg p-3">
      <p class="text-xs text-secondary">{t("Fällig")}</p>
      <p class="font-pixel mt-1 text-sm font-bold text-primary dark:text-primary-dark">{dueCount}</p>
    </div>
  </div>

  <footer class="mt-4 grid grid-cols-2 gap-2 border-t border-current/10 pt-4">
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
