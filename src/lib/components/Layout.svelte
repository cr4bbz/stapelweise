<script lang="ts">
  import ShortcutHelp from "./ShortcutHelp.svelte";
  import LocalizationBridge from "./LocalizationBridge.svelte";
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
<LocalizationBridge />

<div class="h-screen flex flex-col bg-atmosphere transition-colors">
  <main class="flex-1 overflow-hidden min-h-0">
    {@render children()}
  </main>

  <ShortcutHelp visible={showShortcutHelp} onClose={() => (showShortcutHelp = false)} />
</div>
