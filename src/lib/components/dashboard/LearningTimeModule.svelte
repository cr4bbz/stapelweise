<!-- Learning-time module: elapsed study time today and across the current week. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { dashboardStore } from "$lib/stores/dashboard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { buildWeekDays, dateKey, formatDuration, readActivityLog, type ActivityLog } from "$lib/dashboard/activity";
  import { t } from "$lib/i18n";

  let activity = $state<ActivityLog>({});
  let dashboard = $derived(dashboardStore.stats);
  let weekDays = $derived(buildWeekDays(activity, dashboard?.reviews_today ?? 0, settingsStore.current.ui_language));
  let weekSeconds = $derived(weekDays.reduce((sum, day) => sum + day.seconds, 0));
  let todaySeconds = $derived(activity[dateKey(new Date())]?.seconds ?? 0);

  onMount(() => {
    activity = readActivityLog();
  });
</script>

<div class="surface-panel h-full p-5">
  <p class="section-kicker mb-3">{t("Lernzeit")}</p>
  <div class="grid grid-cols-2 gap-3">
    <div class="module-accent-subpanel rounded-lg p-3">
      <p class="text-xs text-secondary">{t("Heute")}</p>
      <p class="font-pixel mt-2 text-sm text-primary dark:text-primary-dark">{formatDuration(todaySeconds)}</p>
    </div>
    <div class="module-accent-subpanel rounded-lg p-3">
      <p class="text-xs text-secondary">{t("Diese Woche")}</p>
      <p class="font-pixel mt-2 text-sm text-primary dark:text-primary-dark">{formatDuration(weekSeconds)}</p>
    </div>
  </div>
</div>
