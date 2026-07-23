<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import { t } from "$lib/i18n";
  let { visible = false, onClose = () => {} } = $props<{
    visible: boolean;
    onClose?: () => void;
  }>();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && visible) onClose();
  }

  const sections = [
    {
      title: "Lernen",
      shortcuts: [
        ["Leertaste", "Karte umdrehen"],
        ["1", "Schwer"],
        ["2", "Mittel"],
        ["3", "Gut"],
        ["4", "Perfekt"],
        ["Strg+Z", "Letzte Bewertung rückgängig"],
        ["Esc", "Session beenden"],
      ],
    },
    {
      title: "Editor",
      shortcuts: [
        ["Strg+Enter", "Karte speichern/erstellen"],
        ["Esc", "Abbrechen / Zurück"],
        ["?", "Diese Hilfe"],
      ],
    },
    {
      title: "Allgemein",
      shortcuts: [
        ["Esc", "Schließen / Zurück"],
        ["?", "Tastenkürzel anzeigen"],
      ],
    },
  ];
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <div in:fade={{ duration: 140 }} out:fade={{ duration: 110 }} class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onclick={onClose} onkeydown={(e) => e.key === "Escape" && onClose()} tabindex="-1" role="presentation">
    <div
      in:scale={{ duration: 180, start: 0.97, opacity: 0 }}
      out:scale={{ duration: 120, start: 0.97, opacity: 0 }}
      class="glass rounded-card p-6 max-w-lg mx-4 shadow-elevation-high max-h-[80vh] overflow-y-auto"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-bold text-primary dark:text-primary-dark">{t("Tastenkürzel")}</h2>
        <button
          onclick={onClose}
          class="p-1 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
          title="Schließen"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>

      {#each sections as section}
        <h3 class="font-semibold text-primary dark:text-primary-dark text-xs uppercase tracking-wide mt-4 mb-2">{t(section.title)}</h3>
        <div class="grid grid-cols-2 gap-2 text-sm">
          {#each section.shortcuts as [keys, desc]}
            <div class="flex items-center gap-2">
              <kbd class="inline-flex items-center justify-center min-w-[1.75rem] h-6 px-1.5 rounded bg-white/50 dark:bg-white/10 border border-white/20 text-xs font-mono text-primary dark:text-primary-dark">{keys}</kbd>
              <span class="text-secondary">{t(desc)}</span>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
{/if}
