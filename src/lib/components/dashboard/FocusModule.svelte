<!-- Focus module: the "learn today" hero with the context-aware primary action
     and a compact stats footer. Falls back to an onboarding state with no data. -->
<script lang="ts">
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { t } from "$lib/i18n";

  let { onRequestNewDeck, onRequestNewExam } = $props<{
    onRequestNewDeck: () => void;
    onRequestNewExam: () => void;
  }>();

  let dashboard = $derived(dashboardStore.stats);
</script>

{#if dashboard}
  <div class="surface-panel h-full overflow-hidden">
    <div class="p-5 sm:p-6">
      <div class="flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <p class="section-kicker mb-2">{navigation.learningLoad}</p>
          <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">{t("todayLearn")}</h1>
        </div>
        <div class="module-accent-soft rounded-lg px-4 py-3 text-left sm:text-right">
          <p class="font-pixel text-2xl font-bold text-primary dark:text-primary-dark">{dashboard.due_cards}</p>
          <p class="text-xs font-semibold uppercase tracking-wide text-secondary">{t("dueCards")}</p>
        </div>
      </div>
      <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
        <button onclick={() => navigation.primaryAction(onRequestNewDeck)} class="primary-action px-5 py-2.5 text-sm">{navigation.primaryActionLabel}</button>
        <button onclick={onRequestNewExam} class="secondary-action px-5 py-2.5 text-sm">{t("Prüfung planen")}</button>
      </div>
    </div>
    <div class="module-accent-soft grid grid-cols-3 border-t">
      <div class="border-r border-current/10 p-4">
        <p class="text-xs font-medium text-secondary">{t("Heute gelernt")}</p>
        <p class="module-accent-text font-pixel mt-2 text-base font-bold">{dashboard.reviews_today}</p>
      </div>
      <div class="border-r border-current/10 p-4">
        <p class="text-xs font-medium text-secondary">{t("Serie")}</p>
        <p class="module-accent-text font-pixel mt-2 text-base font-bold">{dashboard.streak_days} <span class="font-sans text-sm font-semibold text-secondary">{t("Tage")}</span></p>
      </div>
      <div class="p-4">
        <p class="text-xs font-medium text-secondary">{t("Gesamt")}</p>
        <p class="module-accent-text font-pixel mt-2 text-base font-bold">{dashboard.total_cards}</p>
      </div>
    </div>
  </div>
{:else}
  <div class="surface-panel h-full p-5 sm:p-6">
    <p class="section-kicker mb-2">{t("Kleine Runde")}</p>
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark sm:text-3xl">{t("todayLearn")}</h1>
    <div class="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
      <button onclick={onRequestNewDeck} class="primary-action px-5 py-2.5 text-sm">{t("Ersten Stapel anlegen")}</button>
      <button onclick={onRequestNewExam} class="secondary-action px-5 py-2.5 text-sm">{t("Prüfung planen")}</button>
    </div>
  </div>
{/if}
