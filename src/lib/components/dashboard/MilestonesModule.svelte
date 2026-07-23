<!-- Milestones module: gentle, non-gamified progress markers toward review,
     library-size and consistency goals. -->
<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { t } from "$lib/i18n";

  let totalReviews = $state(0);
  let loadedKey = "";
  let dashboard = $derived(dashboardStore.stats);

  let milestones = $derived.by(() => {
    const cards = dashboard?.total_cards ?? 0;
    const streak = dashboard?.streak_days ?? 0;
    return [
      { label: t("Erste Runde"), value: totalReviews, target: 10, suffix: t("Wiederholungen") },
      { label: t("Eigene Bibliothek"), value: cards, target: 50, suffix: t("Karten") },
      { label: t("Im Rhythmus"), value: streak, target: 7, suffix: t("Tage in Folge") },
      { label: t("Fest verankert"), value: totalReviews, target: 100, suffix: t("Wiederholungen") },
    ];
  });

  async function loadTotalReviews() {
    if (deckStore.decks.length === 0) {
      totalReviews = 0;
      return;
    }
    const stats = await Promise.all(deckStore.decks.map((deck) => api.getDeckStats(deck.id)));
    totalReviews = stats.reduce((sum, item) => sum + item.total_reviews_sum, 0);
  }

  $effect(() => {
    const key = `${deckStore.decks.map((deck) => deck.id).join(",")}:${dashboardStore.revision}`;
    if (!key || key === loadedKey) return;
    loadedKey = key;
    void loadTotalReviews();
  });
</script>

<div class="surface-panel h-full p-5">
  <div class="flex items-start justify-between gap-4">
    <p class="section-kicker mb-2">{t("Meilensteine")}</p>
    <span class="module-accent-text font-pixel text-lg">{milestones.filter((item) => item.value >= item.target).length}/{milestones.length}</span>
  </div>
  <div class="mt-5 grid grid-cols-2 gap-3">
    {#each milestones as item}
      {@const complete = item.value >= item.target}
      <div class="module-accent-subpanel rounded-lg p-3">
        <div class="flex items-center justify-between gap-2">
          <p class="text-sm font-semibold text-primary dark:text-primary-dark">{item.label}</p>
          <span class="text-xs font-semibold {complete ? 'module-accent-text' : 'text-secondary'}">{complete ? t("Erreicht") : `${Math.min(item.value, item.target)}/${item.target}`}</span>
        </div>
        <p class="mt-1 text-xs text-secondary">{item.suffix}</p>
        <div class="module-accent-track mt-3 h-1.5 overflow-hidden rounded-full">
          <div class="module-accent-fill h-full rounded-full" style="width: {Math.min(100, (item.value / item.target) * 100)}%"></div>
        </div>
      </div>
    {/each}
  </div>
</div>
