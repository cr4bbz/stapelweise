<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { renderMarkdown } from "$lib/markdown";

  let { front = "", back = "", flipped = false } = $props();

  let renderedFront = $derived(renderMarkdown(front));
  let renderedBack = $derived(renderMarkdown(back));
  let shortCard = $derived(front.length + back.length <= 60);

  let sizeClass = $derived(settingsStore.fontSizeClass(settingsStore.current.card_font_size, shortCard));
  let familyClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));
</script>

<div class="card-flip w-full max-w-2xl mx-auto h-80">
  <div class="card-flip-inner relative w-full h-full {flipped ? 'flipped' : ''}">
    <!-- Front -->
    <div class="card-front absolute inset-0 glass-card rounded-card p-8 flex items-center justify-center shadow-elevation-mid">
      <p class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance select-none">
        {@html renderedFront}
      </p>
    </div>
    <!-- Back -->
    <div class="card-back absolute inset-0 glass-card rounded-card p-8 flex items-center justify-center shadow-elevation-mid">
      <p class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance select-none">
        {@html renderedBack}
      </p>
    </div>
  </div>
</div>
