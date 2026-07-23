// Dashboard statistics store. Holds the aggregate DashboardStats used across
// dashboard modules and exposes a revision counter so child modules can refetch
// their own slices after data changes (a new card, a finished session, …).

import * as api from "$lib/api";
import type { DashboardStats } from "$lib/types";

let stats = $state<DashboardStats | null>(null);
let revision = $state(0);

async function refresh() {
  try {
    stats = await api.getDashboardStats();
  } catch {
    // Keep the previous snapshot if the backend is momentarily unavailable.
  }
}

/** Bump the revision and refetch aggregate stats after a data mutation. */
function invalidate() {
  revision += 1;
  void refresh();
}

export function getDashboardStore() {
  return {
    get stats() {
      return stats;
    },
    get revision() {
      return revision;
    },
    refresh,
    invalidate,
  };
}

const dashboardStore = getDashboardStore();
export { dashboardStore };
