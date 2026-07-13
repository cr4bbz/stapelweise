<script lang="ts">
  import ThemeToggle from "./ThemeToggle.svelte";
  import ShortcutHelp from "./ShortcutHelp.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();
  let showShortcutHelp = $state(false);

  theme.init();

  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === "?" && !e.ctrlKey && !e.metaKey && !e.altKey) {
      // Don't trigger when typing in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      e.preventDefault();
      showShortcutHelp = !showShortcutHelp;
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="h-screen flex flex-col bg-atmosphere transition-colors">
  <!-- Top bar -->
  <header class="border-b border-[#E4E7EC] dark:border-[#2A303B] bg-white/90 dark:bg-[#171B24]/90 backdrop-blur">
    <div class="app-container flex h-14 items-center justify-between">
      <a href="/" class="flex items-center gap-2 text-base font-bold text-primary dark:text-primary-dark tracking-tight">
        <span class="flex h-7 w-7 items-center justify-center rounded-lg bg-accent-correct text-white text-sm font-black">S</span>
        <span>stapelweise</span>
      </a>
      <div class="flex items-center gap-1">
      <button
        class="icon-button"
        title="Suche"
        onclick={() => window.dispatchEvent(new CustomEvent("open-search"))}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
        </svg>
      </button>
      <button
        class="icon-button"
        title="Einstellungen"
        onclick={() => window.dispatchEvent(new CustomEvent("open-settings"))}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
        </svg>
      </button>
      <ThemeToggle />
      </div>
    </div>
  </header>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden min-h-0">
    {@render children()}
  </main>

  <ShortcutHelp visible={showShortcutHelp} onClose={() => (showShortcutHelp = false)} />
</div>
