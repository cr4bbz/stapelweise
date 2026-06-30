<script lang="ts">
  import ThemeToggle from "./ThemeToggle.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  theme.init();
</script>

<div class="h-screen flex flex-col bg-atmosphere transition-colors">
  <!-- Top bar -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-white/10 glass shadow-[0_1px_3px_rgba(0,0,0,0.04)]">
    <a href="/" class="text-lg font-bold text-primary dark:text-primary-dark tracking-tight">
      Stapelweise
    </a>
    <div class="flex items-center gap-1">
      <button
        class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
        title="Einstellungen"
        onclick={() => window.dispatchEvent(new CustomEvent("open-settings"))}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
        </svg>
      </button>
      <ThemeToggle />
    </div>
  </header>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    {@render children()}
  </main>

  <!-- Footer -->
  <footer class="px-6 py-2 text-center text-xs text-secondary border-t border-white/10 glass shadow-[0_-1px_3px_rgba(0,0,0,0.04)]">
    Stapelweise v0.1 &middot; Lerne stapelweise
  </footer>
</div>
