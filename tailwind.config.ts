import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: {
          DEFAULT: "#FAF9F6",
          dark: "#1A1A2E",
        },
        surface: {
          DEFAULT: "#FFFFFF",
          dark: "#16213E",
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
          incorrect: "#DC2626",
          "incorrect-dark": "#EF4444",
        },
        glass: {
          light: "rgba(255, 255, 255, 0.6)",
          dark: "rgba(30, 30, 60, 0.6)",
        },
      },
      fontFamily: {
        sans: ["Inter", "Avenir", "Helvetica", "Arial", "sans-serif"],
        serif: ["Source Serif 4", "Georgia", "serif"],
      },
      borderRadius: {
        card: "1rem",
        button: "9999px",
      },
    },
  },
  plugins: [],
} satisfies Config;
