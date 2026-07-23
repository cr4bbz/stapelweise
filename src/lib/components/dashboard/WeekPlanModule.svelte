<!-- Weekly-plan module: a seven-day review histogram plus a shortcut into
     today's due round. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { buildWeekDays, readActivityLog, type ActivityLog } from "$lib/dashboard/activity";
  import { t } from "$lib/i18n";

  let activity = $state<ActivityLog>({});
  let dashboard = $derived(dashboardStore.stats);
  let weekDays = $derived(buildWeekDays(activity, dashboard?.reviews_today ?? 0, settingsStore.current.ui_language));
  let weekReviews = $derived(weekDays.reduce((sum, day) => sum + day.reviews, 0));

  onMount(() => {
    activity = readActivityLog();
  });
</script>

<div class="surface-panel h-full p-5">
  <div class="flex items-start justify-between gap-4">
    <div>
      <p class="section-kicker mb-2">{t("Wochenplan")}</p>
      <p class="text-sm text-secondary">{weekReviews} {t("Wiederholungen")} {t("in dieser Woche")}</p>
    </div>
    <button onclick={() => navigation.studyToday()} disabled={!dashboard || dashboard.due_cards === 0} class="secondary-action px-3 py-1.5 text-xs disabled:opacity-40">{t("todayLearn")}</button>
  </div>
  <div class="mt-5 grid grid-cols-7 gap-2">
    {#each weekDays as day}
      <div class="flex min-w-0 flex-col items-center gap-2">
        <div class="module-accent-track flex h-16 w-full items-end overflow-hidden rounded">
          <div class="module-accent-fill w-full" style="height: {Math.max(day.reviews > 0 ? 14 : 4, Math.min(100, day.reviews * 7))}%"></div>
        </div>
        <span class="text-[10px] font-semibold {day.today ? 'module-accent-text' : 'text-secondary'}">{day.name}</span>
        <span class="text-xs text-primary dark:text-primary-dark">{day.reviews}</span>
      </div>
    {/each}
  </div>
</div>
