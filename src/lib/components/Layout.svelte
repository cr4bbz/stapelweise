<script lang="ts">
  import ThemeToggle from "./ThemeToggle.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  theme.init();
</script>

<div class="h-screen flex flex-col bg-background dark:bg-background-dark transition-colors">
  <!-- Top bar -->
  <header class="flex items-center justify-between px-6 py-3 border-b border-white/10">
    <a href="/" class="text-lg font-bold text-primary dark:text-primary-dark tracking-tight">
      Stapelweise
    </a>
    <ThemeToggle />
  </header>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    {@render children()}
  </main>

  <!-- Footer -->
  <footer class="px-6 py-2 text-center text-xs text-secondary border-t border-white/10">
    Stapelweise v0.1 &middot; Lerne stapelweise
  </footer>
</div>
