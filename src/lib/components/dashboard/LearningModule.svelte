<!-- Learning-status module: study load, weekly rhythm and library summary. -->
<script lang="ts">
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { t } from "$lib/i18n";

  let dashboard = $derived(dashboardStore.stats);
  let weekDays = $derived(["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"].map((day) => t(day)));
</script>

<aside class="surface-panel h-full p-5">
  <p class="section-kicker mb-3">{t("Lernlage")}</p>
  {#if dashboard}
    <div class="space-y-4">
      <div>
        <div class="mb-1 flex items-center justify-between text-sm">
          <span class="font-semibold text-primary dark:text-primary-dark">{t("Lernpensum")}</span>
          <span class="text-secondary">{dashboard.due_cards === 0 ? t("frei") : dashboard.due_cards <= 20 ? t("normal") : t("hoch")}</span>
        </div>
        <div class="module-accent-track h-2 overflow-hidden rounded-full">
          <div class="module-accent-fill h-full rounded-full transition-all" style="width: {Math.min(100, dashboard.due_cards * 3)}%"></div>
        </div>
      </div>
      <div>
        <div class="mb-2 flex items-center justify-between text-sm">
          <span class="font-semibold text-primary dark:text-primary-dark">{t("Wochenrhythmus")}</span>
          <span class="text-secondary">{dashboard.streak_days > 0 ? t("aktiv") : t("neu starten")}</span>
        </div>
        <div class="grid grid-cols-7 gap-1.5">
          {#each weekDays as day, index}
            {@const active = index >= Math.max(0, 7 - dashboard.streak_days)}
            <div class="flex flex-col items-center gap-1">
              <div class="h-2 w-full rounded-full {active ? 'module-accent-fill' : 'module-accent-track'}"></div>
              <span class="text-[10px] font-semibold text-secondary">{day}</span>
            </div>
          {/each}
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div class="module-accent-subpanel rounded-lg p-3">
          <p class="text-xs text-secondary">Ø Ease</p>
          <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{dashboard.avg_ease_factor.toFixed(2)}</p>
        </div>
        <div class="module-accent-subpanel rounded-lg p-3">
          <p class="text-xs text-secondary">{t("Bibliothek")}</p>
          <p class="font-pixel text-sm font-bold text-primary dark:text-primary-dark">{deckStore.decks.length}</p>
        </div>
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-3 gap-3">
      <div class="module-accent-muted h-16 rounded-lg"></div>
      <div class="module-accent-muted h-16 rounded-lg"></div>
      <div class="module-accent-muted h-16 rounded-lg"></div>
    </div>
  {/if}
</aside>
