import { settingsStore } from "./settings.svelte";

export function getTheme() {
  return {
    get isDark() {
      const t = settingsStore.current.theme;
      if (t === "dark") return true;
      if (t === "light") return false;
      return window.matchMedia("(prefers-color-scheme: dark)").matches;
    },
    toggle() {
      const next = this.isDark ? "light" : "dark";
      settingsStore.save({ theme: next });
    },
    init() {
      settingsStore.load();
    },
  };
}

const theme = getTheme();
export { theme };
