import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: {
          DEFAULT: "#F6F7F9",
          dark: "#10131A",
        },
        surface: {
          DEFAULT: "#FFFFFF",
          dark: "#171B24",
        },
        primary: {
          DEFAULT: "#172033",
          dark: "#F4F6FA",
        },
        secondary: {
          DEFAULT: "#667085",
          dark: "#A8B0BE",
        },
        accent: {
          correct: "#2563EB",
          "correct-dark": "#60A5FA",
          incorrect: "#DC2626",
          "incorrect-dark": "#EF4444",
          easy: "#059669",
          "easy-dark": "#4ADE80",
          hard: "#D97706",
          "hard-dark": "#FBBF24",
        },
        glass: {
          light: "rgba(255, 255, 255, 0.6)",
          dark: "rgba(30, 30, 60, 0.6)",
        },
      },
      fontFamily: {
        sans: ["Source Sans 3", "Avenir", "Helvetica", "Arial", "sans-serif"],
        serif: ["Source Serif 4", "Georgia", "serif"],
        pixel: ["Press Start 2P", "Silkscreen", "monospace"],
      },
      borderRadius: {
        card: "0.75rem",
        button: "0.625rem",
      },
      boxShadow: {
        'elevation-low':  '0 1px 2px rgba(16,24,40,0.04), 0 1px 3px rgba(16,24,40,0.08)',
        'elevation-mid':  '0 4px 10px rgba(16,24,40,0.08), 0 1px 3px rgba(16,24,40,0.06)',
        'elevation-high': '0 12px 28px rgba(16,24,40,0.14), 0 2px 6px rgba(16,24,40,0.08)',
      },
    },
  },
  plugins: [],
} satisfies Config;
