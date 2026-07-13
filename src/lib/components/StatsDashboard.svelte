<script lang="ts">
  import * as api from "$lib/api";
  import type { DeckStats } from "$lib/types";
  import ErrorBanner from "./ErrorBanner.svelte";
  import { fade } from "svelte/transition";

  let { deckId, deckName = "", onClose = () => {} } = $props<{
    deckId: string;
    deckName?: string;
    onClose?: () => void;
  }>();

  let stats = $state<DeckStats | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    loading = true;
    error = null;
    api.getDeckStats(deckId).then((s) => {
      stats = s;
      loading = false;
    }).catch((e) => {
      error = e?.toString() || "Fehler beim Laden der Statistiken";
      loading = false;
    });
  });

  let barTotal = $derived(
    stats ? stats.new_cards + stats.learning_cards + stats.reviewing_cards + stats.mastered_cards : 1
  );

  function barPercent(value: number): number {
    return barTotal > 0 ? Math.round((value / barTotal) * 100) : 0;
  }
</script>

<div class="flex flex-col h-full overflow-y-auto">
  <!-- Header -->
  <div class="flex items-center gap-3 p-6 pb-4">
    <button
      onclick={onClose}
      class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
      title="Zurück zu den Karten"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark truncate">
      {deckName}
    </h1>
    <span class="text-secondary text-sm ml-auto">Statistiken</span>
  </div>

  <!-- Content -->
  <div class="flex-1 px-6 pb-6 grid">
    {#if loading}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex items-center justify-center h-64">
        <p class="text-secondary text-lg">Lade Statistiken...</p>
      </div>
    {:else if error}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1">
        <ErrorBanner message={error} onDismiss={() => (error = null)} />
      </div>
    {:else if stats}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 overflow-y-auto w-full">
        <!-- 2x2 Stat Cards -->
      <div class="grid grid-cols-2 gap-4 mb-8">
        <div class="glass rounded-card p-5">
          <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Gesamt</p>
          <p class="text-3xl font-bold text-primary dark:text-primary-dark">{stats.total_cards}</p>
        </div>
        <div class="glass rounded-card p-5">
          <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Fallig</p>
          <p class="text-3xl font-bold text-accent-correct dark:text-accent-correct-dark">{stats.due_cards}</p>
        </div>
        <div class="glass rounded-card p-5">
          <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">Heute gelernt</p>
          <p class="text-3xl font-bold text-accent-easy dark:text-accent-easy-dark">{stats.reviews_today}</p>
        </div>
        <div class="glass rounded-card p-5">
          <p class="text-xs font-medium text-secondary uppercase tracking-wide mb-1">O Ease</p>
          <p class="text-3xl font-bold text-primary dark:text-primary-dark">{stats.avg_ease_factor.toFixed(2)}</p>
        </div>
      </div>

      <!-- Distribution Bar Chart -->
      <div class="glass rounded-card p-6">
        <h2 class="text-sm font-semibold text-primary dark:text-primary-dark uppercase tracking-wide mb-5">Verteilung</h2>
        <div class="space-y-4">
          <!-- New -->
          <div>
            <div class="flex justify-between text-sm mb-1.5">
              <span class="text-secondary">Neu</span>
              <span class="text-primary dark:text-primary-dark font-medium">{stats.new_cards} ({barPercent(stats.new_cards)}%)</span>
            </div>
            <div class="h-3 bg-white/20 dark:bg-white/5 rounded-full overflow-hidden">
              <div class="h-full bg-blue-400/70 rounded-full transition-all duration-500" style="width: {barPercent(stats.new_cards)}%"></div>
            </div>
          </div>

          <!-- Learning -->
          <div>
            <div class="flex justify-between text-sm mb-1.5">
              <span class="text-secondary">Lernend</span>
              <span class="text-primary dark:text-primary-dark font-medium">{stats.learning_cards} ({barPercent(stats.learning_cards)}%)</span>
            </div>
            <div class="h-3 bg-white/20 dark:bg-white/5 rounded-full overflow-hidden">
              <div class="h-full bg-accent-hard rounded-full transition-all duration-500" style="width: {barPercent(stats.learning_cards)}%"></div>
            </div>
          </div>

          <!-- Reviewing -->
          <div>
            <div class="flex justify-between text-sm mb-1.5">
              <span class="text-secondary">Wiederholend</span>
              <span class="text-primary dark:text-primary-dark font-medium">{stats.reviewing_cards} ({barPercent(stats.reviewing_cards)}%)</span>
            </div>
            <div class="h-3 bg-white/20 dark:bg-white/5 rounded-full overflow-hidden">
              <div class="h-full bg-accent-easy rounded-full transition-all duration-500" style="width: {barPercent(stats.reviewing_cards)}%"></div>
            </div>
          </div>

          <!-- Mastered -->
          <div>
            <div class="flex justify-between text-sm mb-1.5">
              <span class="text-secondary">Gelernt</span>
              <span class="text-primary dark:text-primary-dark font-medium">{stats.mastered_cards} ({barPercent(stats.mastered_cards)}%)</span>
            </div>
            <div class="h-3 bg-white/20 dark:bg-white/5 rounded-full overflow-hidden">
              <div class="h-full bg-accent-correct rounded-full transition-all duration-500" style="width: {barPercent(stats.mastered_cards)}%"></div>
            </div>
          </div>
        </div>

        <!-- Extra stats row -->
        <div class="mt-6 pt-4 border-t border-white/10 grid grid-cols-3 gap-4 text-xs">
          <div>
            <span class="text-secondary">O Intervall</span>
            <p class="text-primary dark:text-primary-dark font-medium mt-0.5">{stats.avg_interval.toFixed(1)} Tage</p>
          </div>
          <div>
            <span class="text-secondary">Reviews gesamt</span>
            <p class="text-primary dark:text-primary-dark font-medium mt-0.5">{stats.total_reviews_sum}</p>
          </div>
          <div>
            <span class="text-secondary">Gelernt-Quote</span>
            <p class="text-primary dark:text-primary-dark font-medium mt-0.5">
              {stats.total_cards > 0 ? Math.round((stats.mastered_cards / stats.total_cards) * 100) : 0}%
            </p>
          </div>
        </div>
      </div>
      </div>
    {/if}
  </div>
</div>
