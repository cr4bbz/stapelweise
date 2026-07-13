<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { renderMarkdown } from "$lib/markdown";

  let { front = "", back = "", reasoning = null, tags = [], flipped = false } = $props<{
    front?: string;
    back?: string;
    reasoning?: string | null;
    tags?: string[];
    flipped?: boolean;
  }>();

  let isCloze = $derived(front.includes("==") || front.includes("{{c1::"));

  function renderClozeFront(text: string) {
    let res = text.replace(/==(.*?)==/g, "[...]");
    res = res.replace(/\{\{c1::(.*?)\}\}/g, "[...]");
    return renderMarkdown(res);
  }

  function renderClozeBack(text: string) {
    let res = text.replace(/==(.*?)==/g, '<span class="text-accent-correct bg-accent-correct/10 px-1 rounded font-bold">$1</span>');
    res = res.replace(/\{\{c1::(.*?)\}\}/g, '<span class="text-accent-correct bg-accent-correct/10 px-1 rounded font-bold">$1</span>');
    return renderMarkdown(res);
  }

  let renderedFront = $derived(isCloze ? renderClozeFront(front) : renderMarkdown(front));
  let renderedBack = $derived(isCloze ? renderClozeBack(front) + (back.trim() ? `<hr class="my-4 border-white/10"/>${renderMarkdown(back)}` : "") : renderMarkdown(back));
  let renderedReasoning = $derived(reasoning ? renderMarkdown(reasoning) : null);
  let shortCard = $derived(front.length + back.length + (reasoning?.length || 0) <= 60);

  let sizeClass = $derived(settingsStore.fontSizeClass(settingsStore.current.card_font_size, shortCard));
  let familyClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));
</script>

<div class="card-flip w-full max-w-2xl mx-auto h-80">
  <div class="card-flip-inner relative w-full h-full {flipped ? 'flipped' : ''}">
    <!-- Front -->
    <div class="card-front absolute inset-0 glass-card rounded-card p-8 flex flex-col items-center justify-center shadow-elevation-mid">
      {#if tags.length > 0}
        <div class="absolute top-4 left-4 right-4 flex flex-wrap gap-1 justify-center">
          {#each tags as tag}
            <span class="inline-flex items-center bg-white/10 text-secondary px-2 py-0.5 rounded text-[10px] font-medium tracking-wide">#{tag}</span>
          {/each}
        </div>
      {/if}
      <div class="flex-1 flex items-center justify-center w-full mt-4">
        <p class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance select-none">
          {@html renderedFront}
        </p>
      </div>
    </div>
    <!-- Back -->
    <div class="card-back absolute inset-0 glass-card rounded-card p-8 flex flex-col items-center justify-center shadow-elevation-mid overflow-y-auto">
      {#if tags.length > 0}
        <div class="absolute top-4 left-4 right-4 flex flex-wrap gap-1 justify-center">
          {#each tags as tag}
            <span class="inline-flex items-center bg-white/10 text-secondary px-2 py-0.5 rounded text-[10px] font-medium tracking-wide">#{tag}</span>
          {/each}
        </div>
      {/if}
      <div class="flex-1 flex items-center justify-center w-full mt-4">
        <div class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance select-none w-full">
          {@html renderedBack}
        </div>
      </div>
      {#if renderedReasoning}
        <div class="mt-4 pt-4 border-t border-white/10 w-full text-center shrink-0">
          <span class="text-[10px] uppercase text-secondary/70 font-semibold tracking-wider">Warum?</span>
          <div class="{familyClass} text-sm mt-1 text-primary/80 dark:text-primary-dark/80 text-balance select-none opacity-90">
            {@html renderedReasoning}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
