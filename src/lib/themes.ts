export const colorThemes = [
  { id: "academy", label: "Akademie", primary: "#2563eb", secondary: "#059669" },
  { id: "night-library", label: "Nachtbibliothek", primary: "#0f766e", secondary: "#2563eb" },
  { id: "printwork", label: "Druckwerk", primary: "#be123c", secondary: "#0f766e" },
  { id: "graphite", label: "Graphit", primary: "#475569", secondary: "#0891b2" },
] as const;

export type ColorTheme = (typeof colorThemes)[number]["id"];

export function isColorTheme(value: string): value is ColorTheme {
  return colorThemes.some((theme) => theme.id === value);
}
