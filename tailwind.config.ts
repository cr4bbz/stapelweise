import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: {
          DEFAULT: "rgb(from var(--color-bg) r g b / <alpha-value>)",
          dark: "rgb(from var(--color-bg) r g b / <alpha-value>)",
        },
        surface: {
          DEFAULT: "rgb(from var(--color-surface) r g b / <alpha-value>)",
          dark: "rgb(from var(--color-surface) r g b / <alpha-value>)",
        },
        primary: {
          DEFAULT: "rgb(from var(--color-text) r g b / <alpha-value>)",
          dark: "rgb(from var(--color-text) r g b / <alpha-value>)",
        },
        secondary: {
          DEFAULT: "rgb(from var(--color-text-secondary) r g b / <alpha-value>)",
          dark: "rgb(from var(--color-text-secondary) r g b / <alpha-value>)",
        },
        accent: {
          correct: "rgb(var(--color-accent-primary) / <alpha-value>)",
          "correct-dark": "rgb(var(--color-accent-primary) / <alpha-value>)",
          incorrect: "rgb(var(--color-accent-danger) / <alpha-value>)",
          "incorrect-dark": "rgb(var(--color-accent-danger) / <alpha-value>)",
          easy: "rgb(var(--color-accent-secondary) / <alpha-value>)",
          "easy-dark": "rgb(var(--color-accent-secondary) / <alpha-value>)",
          hard: "rgb(var(--color-accent-warning) / <alpha-value>)",
          "hard-dark": "rgb(var(--color-accent-warning) / <alpha-value>)",
        },
        glass: {
          light: "var(--color-glass)",
          dark: "var(--color-glass)",
        },
      },
      fontFamily: {
        sans: ["Source Sans 3", "Avenir", "Helvetica", "Arial", "sans-serif"],
        serif: ["Source Serif 4", "Georgia", "serif"],
        pixel: ["var(--font-pixel)"],
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
