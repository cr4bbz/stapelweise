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

export type ColorSuggestion = { label: string; color: string };

function hexToHsl(hex: string): [number, number, number] {
  const value = hex.replace("#", "");
  const red = Number.parseInt(value.slice(0, 2), 16) / 255;
  const green = Number.parseInt(value.slice(2, 4), 16) / 255;
  const blue = Number.parseInt(value.slice(4, 6), 16) / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const lightness = (max + min) / 2;
  const delta = max - min;
  if (!delta) return [0, 0, lightness * 100];
  const saturation = delta / (1 - Math.abs(2 * lightness - 1));
  const hue = max === red
    ? 60 * (((green - blue) / delta) % 6)
    : max === green
      ? 60 * ((blue - red) / delta + 2)
      : 60 * ((red - green) / delta + 4);
  return [(hue + 360) % 360, saturation * 100, lightness * 100];
}

function hslToHex(hue: number, saturation: number, lightness: number): string {
  const normalizedHue = ((hue % 360) + 360) % 360;
  const chroma = (1 - Math.abs(2 * lightness / 100 - 1)) * saturation / 100;
  const segment = normalizedHue / 60;
  const middle = chroma * (1 - Math.abs(segment % 2 - 1));
  const [red, green, blue] = segment < 1 ? [chroma, middle, 0]
    : segment < 2 ? [middle, chroma, 0]
      : segment < 3 ? [0, chroma, middle]
        : segment < 4 ? [0, middle, chroma]
          : segment < 5 ? [middle, 0, chroma]
            : [chroma, 0, middle];
  const match = lightness / 100 - chroma / 2;
  return `#${[red, green, blue]
    .map((channel) => Math.round((channel + match) * 255).toString(16).padStart(2, "0"))
    .join("")}`;
}

/** Harmonious, clearly distinguishable companion colours for a selected base colour. */
export function colorSuggestions(baseColor: string): ColorSuggestion[] {
  if (!/^#[0-9a-f]{6}$/i.test(baseColor)) return [];
  const [hue, saturation, lightness] = hexToHsl(baseColor);
  const balancedLightness = lightness > 58 ? 42 : 56;
  const balancedSaturation = Math.max(48, Math.min(78, saturation || 58));
  return [
    { label: "Komplementär", color: hslToHex(hue + 180, balancedSaturation, balancedLightness) },
    { label: "Kühler Akzent", color: hslToHex(hue + 145, balancedSaturation, balancedLightness) },
    { label: "Warmer Akzent", color: hslToHex(hue + 35, balancedSaturation, balancedLightness) },
  ];
}
