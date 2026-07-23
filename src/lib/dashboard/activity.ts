// Shared helpers for the local learning-activity log that several dashboard
// modules read (weekly plan, learning time). The log lives entirely in
// localStorage and maps a YYYY-MM-DD key to review count and elapsed seconds.

import { t } from "$lib/i18n";

export type ActivityDay = { reviews: number; seconds: number };
export type ActivityLog = Record<string, ActivityDay>;
export type WeekDay = { name: string; key: string; reviews: number; seconds: number; today: boolean };

const activityStorageKey = "stapelweise.learning.activity.v1";

export function dateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function readActivityLog(): ActivityLog {
  try {
    return JSON.parse(localStorage.getItem(activityStorageKey) ?? "{}") as ActivityLog;
  } catch {
    return {};
  }
}

/**
 * Build the seven days of the current week (Mon–Sun) with review/second totals.
 * Today's review count is lifted to at least `reviewsToday` so the dashboard's
 * live counter is reflected even before the session is written to the log.
 */
export function buildWeekDays(activity: ActivityLog, reviewsToday: number, uiLanguage: string): WeekDay[] {
  const today = new Date();
  const monday = new Date(today);
  const weekday = today.getDay() || 7;
  monday.setDate(today.getDate() - weekday + 1);
  const formatter = new Intl.DateTimeFormat(uiLanguage, { weekday: "short" });
  const todayKey = dateKey(today);

  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(monday);
    date.setDate(monday.getDate() + index);
    const key = dateKey(date);
    const storedReviews = activity[key]?.reviews ?? 0;
    const reviews = key === todayKey ? Math.max(storedReviews, reviewsToday) : storedReviews;
    return { name: formatter.format(date), key, reviews, seconds: activity[key]?.seconds ?? 0, today: key === todayKey };
  });
}

export function formatDuration(seconds: number): string {
  if (seconds < 60) return `< 1 ${t("Min.")}`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} ${t("Min.")}`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest > 0 ? `${hours} ${t("Std.")} ${rest} ${t("Min.")}` : `${hours} ${t("Std.")}`;
}
