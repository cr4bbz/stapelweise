import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: {
          DEFAULT: "#FAF9F6",
          dark: "#0B0B13",
        },
        surface: {
          DEFAULT: "#FFFFFF",
          dark: "#12121F",
        },
        primary: {
          DEFAULT: "#1E3A5F",
          dark: "#E0E0E0",
        },
        secondary: {
          DEFAULT: "#64748B",
          dark: "#94A3B8",
        },
        accent: {
          correct: "#E6A817",
          "correct-dark": "#FACC15",
          incorrect: "#DC2626",
          "incorrect-dark": "#EF4444",
          easy: "#22C55E",
          "easy-dark": "#4ADE80",
          hard: "#F97316",
          "hard-dark": "#FB923C",
        },
        glass: {
          light: "rgba(255, 255, 255, 0.6)",
          dark: "rgba(30, 30, 60, 0.6)",
        },
      },
      fontFamily: {
        sans: ["Source Sans 3", "Avenir", "Helvetica", "Arial", "sans-serif"],
        serif: ["Source Serif 4", "Georgia", "serif"],
      },
      borderRadius: {
        card: "1rem",
        button: "9999px",
      },
      boxShadow: {
        'elevation-low':  '0 1px 2px rgba(0,0,0,0.04), 0 2px 4px rgba(0,0,0,0.04)',
        'elevation-mid':  '0 2px 4px rgba(0,0,0,0.04), 0 4px 12px rgba(0,0,0,0.08)',
        'elevation-high': '0 4px 8px rgba(0,0,0,0.04), 0 8px 24px rgba(0,0,0,0.12)',
      },
    },
  },
  plugins: [],
} satisfies Config;
