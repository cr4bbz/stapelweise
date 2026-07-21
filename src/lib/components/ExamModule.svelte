<script lang="ts">
  import { Archive, FlaskConical, Pencil, Play } from "@lucide/svelte";
  import * as api from "$lib/api";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
  import type { Exam, ExamStats } from "$lib/types";

  let {
    exam,
    refreshToken = 0,
    onStudy = (_exam: Exam) => {},
    onSimulate = (_exam: Exam) => {},
    onEdit = (_exam: Exam) => {},
    onArchive = (_exam: Exam) => {},
  } = $props<{
    exam: Exam;
    refreshToken?: number;
    onStudy?: (exam: Exam) => void;
    onSimulate?: (exam: Exam) => void;
    onEdit?: (exam: Exam) => void;
    onArchive?: (exam: Exam) => void;
  }>();

  let stats = $state<ExamStats | null>(null);
  let hasStatsError = $state(false);
  let loadedKey = $state("");
  let daysLeft = $derived(stats?.days_left ?? null);
  let dateLabel = $derived(new Intl.DateTimeFormat(settingsStore.current.ui_language).format(new Date(`${exam.exam_date}T00:00:00`)));

  $effect(() => {
    const requestKey = `${exam.id}:${refreshToken}`;
    if (requestKey === loadedKey) return;
    loadedKey = requestKey;
    hasStatsError = false;
    api.getExamStats(exam.id)
      .then((next) => (stats = next))
      .catch(() => {
        stats = null;
        hasStatsError = true;
      });
  });

  function urgencyLabel() {
    if (hasStatsError) return t("Daten nicht verfügbar");
    if (daysLeft === null) return t("wird berechnet");
    if (daysLeft <= 3) return daysLeft === 0 ? t("heute") : t("dringend");
    if (daysLeft <= 14) return t("diese Woche");
    return t("geplant");
  }

  let urgencyClass = $derived(daysLeft === null
    ? hasStatsError
      ? "border-accent-incorrect/45 bg-accent-incorrect/10 text-accent-incorrect"
      : "border-current/20 bg-transparent text-secondary"
    : daysLeft <= 3
      ? "border-accent-incorrect/45 bg-accent-incorrect/10 text-accent-incorrect"
      : daysLeft <= 14
        ? "border-accent-hard/45 bg-accent-hard/10 text-accent-hard"
        : "border-accent-correct/45 bg-accent-correct/10 text-accent-correct");
</script>

<article class="surface-panel flex h-full min-h-56 flex-col overflow-hidden p-4 transition-colors hover:border-accent-hard/45">
  <header class="flex items-start justify-between gap-3">
    <div class="min-w-0">
      <p class="section-kicker mb-2">{t("Pr\u00fcfung")}</p>
      <h2 data-user-content class="line-clamp-2 text-xl leading-tight text-primary dark:text-primary-dark">{exam.name}</h2>
      <p class="mt-1 text-xs text-secondary">{dateLabel}</p>
    </div>
    <div class="flex shrink-0 gap-1">
      <button class="icon-button !h-8 !w-8" onclick={() => onEdit(exam)} title={t("Bearbeiten")} aria-label={t("Bearbeiten")}><Pencil size={15} /></button>
      <button class="icon-button !h-8 !w-8" onclick={() => onArchive(exam)} title={t("Pr\u00fcfung archivieren")} aria-label={t("Pr\u00fcfung archivieren")}><Archive size={15} /></button>
    </div>
  </header>

  <div class="mt-5 grid grid-cols-[auto_1fr] items-center gap-3">
    <div class="rounded-lg border px-3 py-2 text-center {urgencyClass}">
      <p class="text-[10px] font-semibold uppercase">{exam.exam_type}</p>
      <p class="font-pixel mt-1 text-lg font-bold leading-none">{stats ? daysLeft : "-"}</p>
      <p class="mt-1 text-[10px] font-semibold uppercase">{t("Tage")}</p>
    </div>
    <div class="min-w-0">
      <p class="text-xs text-secondary">{urgencyLabel()}</p>
      <p class="mt-1 text-sm font-semibold text-primary dark:text-primary-dark">
        {stats ? `${stats.cards_left} / ${stats.total_cards}` : hasStatsError ? t("Daten nicht verfügbar") : "..."} {stats || hasStatsError ? t("Karten") : ""}
      </p>
      <p class="mt-1 text-xs text-secondary">{stats ? `${stats.cards_per_day} ${t("Karten")} / ${t("Tage")}` : ""}</p>
    </div>
  </div>

  <footer class="mt-auto grid grid-cols-2 gap-2 pt-5">
    <button class="primary-action flex items-center justify-center gap-2 px-3 py-2 text-xs" onclick={() => onStudy(exam)} disabled={exam.deck_ids.length === 0}>
      <Play size={15} /> {t("Lernen")}
    </button>
    <button class="secondary-action flex items-center justify-center gap-2 px-3 py-2 text-xs" onclick={() => onSimulate(exam)} disabled={exam.deck_ids.length === 0} title={t("Simulation")}>
      <FlaskConical size={15} /> {t("Simulation")}
    </button>
  </footer>
</article>
