import * as api from "$lib/api";
import { isColorTheme } from "$lib/themes";
import type { AppSettings, ModuleColorSlot, ModuleColorTarget } from "$lib/types";

const defaultModuleColorAssignments: Record<ModuleColorTarget, ModuleColorSlot> = {
  brand: "primary",
  deck: "primary",
  single_card: "primary",
  timer: "secondary",
  tags: "primary",
  settings: "secondary",
  archive: "secondary",
  exam: "secondary",
};

const defaults: AppSettings = {
  ui_language: "de",
  theme: "light",
  color_theme: "academy",
  custom_primary_color: "#2563eb",
  custom_secondary_color: "#059669",
  module_color_assignments: JSON.stringify(defaultModuleColorAssignments),
  module_surface: "glass",
  show_deck_card_previews: true,
  pixel_font: "press-start",
  card_font_family: "serif",
  card_font_size: "medium",
  learning_animations: true,
  card_flip_animation: true,
  control_transition_animation: true,
  rating_buttons_animation: true,
  session_limit: 50,
  timer_slider_scale: "logarithmic",
  timer_min_minutes: 5,
  timer_max_minutes: 240,
  sm2_initial_ef: 2.5,
  sm2_pass_threshold: 3,
  obsidian_vault_path: "",
  obsidian_flashcard_tag: "#flashcard",
};

const colorThemeStorageKey = "stapelweise.color-theme";
const pixelFontStorageKey = "stapelweise.pixel-font";

function storedColorTheme(): AppSettings["color_theme"] {
  if (typeof localStorage === "undefined") return defaults.color_theme;
  const savedTheme = localStorage.getItem(colorThemeStorageKey);
  return savedTheme === "custom" || (savedTheme && isColorTheme(savedTheme)) ? savedTheme : defaults.color_theme;
}

function storedPixelFont(): AppSettings["pixel_font"] {
  if (typeof localStorage === "undefined") return defaults.pixel_font;
  const savedFont = localStorage.getItem(pixelFontStorageKey);
  return savedFont === "silkscreen" || savedFont === "press-start" || savedFont === "source-sans" || savedFont === "source-serif"
    ? savedFont
    : defaults.pixel_font;
}

const initialColorTheme = storedColorTheme();
const initialPixelFont = storedPixelFont();
let current = $state<AppSettings>({
  ...defaults,
  color_theme: initialColorTheme,
  pixel_font: initialPixelFont,
});
let loaded = $state(false);
let loadPromise: Promise<void> | null = null;

function applyThemeToDom() {
  if (typeof document === "undefined") return;
  document.documentElement.classList.remove("dark");
  localStorage.setItem("theme", "light");
}

function isHexColor(value: string): boolean {
  return /^#[0-9a-f]{6}$/i.test(value);
}

function hexToRgbValue(hex: string): string {
  const value = hex.replace("#", "");
  return `${Number.parseInt(value.slice(0, 2), 16)} ${Number.parseInt(value.slice(2, 4), 16)} ${Number.parseInt(value.slice(4, 6), 16)}`;
}

function applyColorThemeToDom(
  colorTheme: AppSettings["color_theme"],
  primaryColor = current.custom_primary_color,
  secondaryColor = current.custom_secondary_color,
) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const resolvedTheme = isColorTheme(colorTheme) ? colorTheme : "custom";
  root.dataset.colorTheme = resolvedTheme;
  localStorage.setItem(colorThemeStorageKey, resolvedTheme);
  for (const property of ["--color-accent-primary", "--color-accent-secondary", "--dashboard-tone-primary", "--dashboard-tone-secondary", "--dashboard-tone-warm"]) {
    root.style.removeProperty(property);
  }
  if (colorTheme === "custom" && isHexColor(primaryColor) && isHexColor(secondaryColor)) {
    const primary = hexToRgbValue(primaryColor);
    const secondary = hexToRgbValue(secondaryColor);
    root.style.setProperty("--color-accent-primary", primary);
    root.style.setProperty("--color-accent-secondary", secondary);
    root.style.setProperty("--dashboard-tone-primary", primary);
    root.style.setProperty("--dashboard-tone-secondary", secondary);
    root.style.setProperty("--dashboard-tone-warm", secondary);
  }
}

function parsedModuleColorAssignments(value: string): Record<ModuleColorTarget, ModuleColorSlot> {
  try {
    const parsed = JSON.parse(value) as Partial<Record<ModuleColorTarget, unknown>>;
    return Object.fromEntries(
      Object.entries(defaultModuleColorAssignments).map(([target, fallback]) => [
        target,
        parsed[target as ModuleColorTarget] === "secondary" ? "secondary" : parsed[target as ModuleColorTarget] === "primary" ? "primary" : fallback,
      ])
    ) as Record<ModuleColorTarget, ModuleColorSlot>;
  } catch {
    return defaultModuleColorAssignments;
  }
}

function applyModuleSurfaceToDom() {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.moduleSurface = "glass";
  localStorage.setItem("stapelweise.module-surface", "glass");
}

function applyPixelFontToDom(pixelFont: AppSettings["pixel_font"]) {
  if (typeof document === "undefined") return;
  const resolvedFont = ["press-start", "silkscreen", "source-sans", "source-serif"].includes(pixelFont)
    ? pixelFont
    : "press-start";
  document.documentElement.dataset.pixelFont = resolvedFont;
  localStorage.setItem(pixelFontStorageKey, resolvedFont);
}

if (typeof document !== "undefined") {
  applyThemeToDom();
  applyColorThemeToDom(initialColorTheme);
  applyModuleSurfaceToDom();
  applyPixelFontToDom(initialPixelFont);
}

function applyLanguageToDom(language: AppSettings["ui_language"]) {
  if (typeof document === "undefined") return;
  document.documentElement.lang = language;
}

async function load() {
  if (loaded) return;
  if (loadPromise) return loadPromise;
  loadPromise = (async () => {
    try {
      const s = await api.getSettings();
      current = { ...defaults, ...s, theme: "light", module_surface: "glass" };
    } catch {
      // Use defaults if backend isn't ready
    } finally {
      loaded = true;
      loadPromise = null;
      applyThemeToDom();
      applyColorThemeToDom(current.color_theme, current.custom_primary_color, current.custom_secondary_color);
      applyModuleSurfaceToDom();
      applyPixelFontToDom(current.pixel_font);
      applyLanguageToDom(current.ui_language);
    }
  })();
  return loadPromise;
}

async function save(partial: Partial<AppSettings>) {
  current = { ...current, ...partial };
  if ("theme" in partial) applyThemeToDom();
  if ("color_theme" in partial || "custom_primary_color" in partial || "custom_secondary_color" in partial) {
    applyColorThemeToDom(current.color_theme, current.custom_primary_color, current.custom_secondary_color);
  }
  if ("module_surface" in partial) applyModuleSurfaceToDom();
  if ("pixel_font" in partial) applyPixelFontToDom(partial.pixel_font!);
  if ("ui_language" in partial) applyLanguageToDom(partial.ui_language!);
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

function animationEnabled(setting: boolean): boolean {
  if (!current.learning_animations || !setting) return false;
  return typeof window === "undefined" || !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function cardFlipAnimationEnabled(): boolean {
  return animationEnabled(current.card_flip_animation);
}

function controlTransitionAnimationEnabled(): boolean {
  return animationEnabled(current.control_transition_animation);
}

function ratingButtonsAnimationEnabled(): boolean {
  return animationEnabled(current.rating_buttons_animation);
}

function moduleColorFor(target: ModuleColorTarget): ModuleColorSlot {
  return parsedModuleColorAssignments(current.module_color_assignments)[target];
}

function setModuleColor(target: ModuleColorTarget, color: ModuleColorSlot) {
  const assignments = parsedModuleColorAssignments(current.module_color_assignments);
  if (assignments[target] === color) return;
  void save({ module_color_assignments: JSON.stringify({ ...assignments, [target]: color }) });
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
    moduleColorFor,
    setModuleColor,
    cardFlipAnimationEnabled,
    controlTransitionAnimationEnabled,
    ratingButtonsAnimationEnabled,
  };
}

const settingsStore = getSettingsStore();
export { settingsStore };
