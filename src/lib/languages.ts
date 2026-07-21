export const languageOptions = [
  { code: "de", label: "Deutsch" },
  { code: "en", label: "Englisch" },
  { code: "fr", label: "Französisch" },
  { code: "es", label: "Spanisch" },
  { code: "it", label: "Italienisch" },
  { code: "pt", label: "Portugiesisch" },
  { code: "nl", label: "Niederländisch" },
  { code: "pl", label: "Polnisch" },
  { code: "tr", label: "Türkisch" },
  { code: "ru", label: "Russisch" },
  { code: "uk", label: "Ukrainisch" },
  { code: "ar", label: "Arabisch" },
  { code: "zh", label: "Chinesisch" },
  { code: "ja", label: "Japanisch" },
  { code: "ko", label: "Koreanisch" },
  { code: "la", label: "Latein" },
  { code: "el", label: "Griechisch" },
] as const;

export function languageLabel(code: string | null | undefined): string {
  if (!code) return "";
  const locale = settingsStore.current.ui_language || "de";

  try {
    return new Intl.DisplayNames([locale], { type: "language" }).of(code) ?? code.toUpperCase();
  } catch {
    return code.toUpperCase();
  }
}
import { settingsStore } from "$lib/stores/settings.svelte";
