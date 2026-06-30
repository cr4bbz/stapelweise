import * as api from "$lib/api";
import type { AppSettings } from "$lib/types";

const defaults: AppSettings = {
  theme: "auto",
  card_font_family: "serif",
  card_font_size: "medium",
  session_limit: 50,
  sm2_initial_ef: 2.5,
  sm2_pass_threshold: 3,
};

let current = $state<AppSettings>({ ...defaults });
let loaded = $state(false);
let loadPromise: Promise<void> | null = null;

function applyThemeToDom(theme: string) {
  if (typeof document === "undefined") return;
  let isDark: boolean;
  if (theme === "dark") {
    isDark = true;
  } else if (theme === "light") {
    isDark = false;
  } else {
    isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  document.documentElement.classList.toggle("dark", isDark);
  localStorage.setItem("theme", isDark ? "dark" : "light");
}

async function load() {
  if (loaded) return;
  if (loadPromise) return loadPromise;
  loadPromise = (async () => {
    try {
      const s = await api.getSettings();
      current = { ...defaults, ...s };
    } catch {
      // Use defaults if backend isn't ready
    } finally {
      loaded = true;
      loadPromise = null;
      applyThemeToDom(current.theme);
    }
  })();
  return loadPromise;
}

async function save(partial: Partial<AppSettings>) {
  current = { ...current, ...partial };
  if ("theme" in partial) applyThemeToDom(partial.theme!);
  try {
    await api.updateSettings(current);
  } catch (e) {
    console.error("[stapelweise] Failed to save settings:", e);
  }
}

/** Mapped font-size classes for FlashCard */
function fontSizeClass(fontSize: AppSettings["card_font_size"], shortCard: boolean): string {
  const map: Record<string, [string, string]> = {
    small: ["text-2xl", "text-xl"],
    medium: ["text-3xl", "text-2xl"],
    large: ["text-4xl", "text-3xl"],
  };
  const [large, small] = map[fontSize] ?? map.medium;
  return shortCard ? large : small;
}

/** Mapped font-family class for FlashCard */
function fontFamilyClass(family: AppSettings["card_font_family"]): string {
  return family === "sans" ? "font-card-sans" : "font-card";
}

export function getSettingsStore() {
  return {
    get current() {
      return current;
    },
    get loaded() {
      return loaded;
    },
    load,
    save,
    fontSizeClass,
    fontFamilyClass,
  };
}

const settingsStore = getSettingsStore();
export { settingsStore };
