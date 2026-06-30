let isDark = $state(false);

export function getTheme() {
  return {
    get isDark() {
      return isDark;
    },
    set isDark(v: boolean) {
      isDark = v;
      if (typeof document !== "undefined") {
        document.documentElement.classList.toggle("dark", v);
        localStorage.setItem("theme", v ? "dark" : "light");
      }
    },
    toggle() {
      this.isDark = !isDark;
    },
    init() {
      const stored = localStorage.getItem("theme");
      if (stored) {
        this.isDark = stored === "dark";
      } else {
        // Respect OS preference
        this.isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      }
    },
  };
}

const theme = getTheme();
export { theme };
