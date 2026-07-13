<script lang="ts">
  import { onMount } from "svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { renderMarkdown } from "$lib/markdown";

  let {
    front = "",
    back = "",
    reasoning = null,
    tags = [],
    flipped = false,
    cardType = "basic",
    content = null,
  } = $props<{
    front?: string;
    back?: string;
    reasoning?: string | null;
    tags?: string[];
    flipped?: boolean;
    cardType?: string;
    content?: string | null;
  }>();

  let zoomedImageSrc = $state<string | null>(null);

  onMount(() => {
    const handleZoom = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (typeof detail === "string") {
        zoomedImageSrc = detail;
      }
    };
    window.addEventListener("stapelweise:zoom-image", handleZoom);
    return () => {
      window.removeEventListener("stapelweise:zoom-image", handleZoom);
    };
  });

  let isCloze = $derived(cardType === "cloze" || front.includes("==") || front.includes("{{c1::"));
  let isMultipleChoice = $derived(cardType === "multiple_choice");
  let isOrdering = $derived(cardType === "ordering");

  // Parse MC Options from JSON content or Markdown back
  interface McOption {
    text: string;
    correct: boolean;
  }

  let mcOptions = $derived.by<McOption[]>(() => {
    if (content) {
      try {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed?.options)) return parsed.options;
      } catch {}
    }
    // Fallback parse from back markdown format: [x] Text or [ ] Text
    const lines = back.split("\n");
    const options: McOption[] = [];
    for (const line of lines) {
      const match = line.match(/^\s*\[([ xX])\]\s*(.*)$/);
      if (match) {
        options.push({
          correct: match[1].toLowerCase() === "x",
          text: match[2].trim(),
        });
      }
    }
    return options;
  });

  // User interactive state for MC (selected indices)
  let selectedMcIndices = $state<number[]>([]);

  $effect(() => {
    // Reset selections when card changes
    front;
    content;
    selectedMcIndices = [];
  });

  function toggleMcOption(idx: number, e: MouseEvent) {
    e.stopPropagation(); // prevent flipping card when clicking option
    if (selectedMcIndices.includes(idx)) {
      selectedMcIndices = selectedMcIndices.filter(i => i !== idx);
    } else {
      selectedMcIndices = [...selectedMcIndices, idx];
    }
  }

  // Parse Ordering items from JSON content or Markdown lines
  let originalOrderingItems = $derived.by<string[]>(() => {
    if (content) {
      try {
        const parsed = JSON.parse(content);
        if (Array.isArray(parsed?.items)) return parsed.items;
      } catch {}
    }
    // Fallback parse from back markdown lines (e.g. 1. Step A)
    return back
      .split("\n")
      .map((l: string) => l.replace(/^\d+\.\s*/, "").trim())
      .filter(Boolean);
  });

  // Shuffled items for interactive front
  let userOrderingItems = $state<string[]>([]);

  $effect(() => {
    if (isOrdering && originalOrderingItems.length > 0) {
      // Proper random shuffle per card load
      userOrderingItems = [...originalOrderingItems].sort(() => Math.random() - 0.5);
    }
  });

  function moveItem(index: number, direction: -1 | 1, e: MouseEvent) {
    e.stopPropagation();
    const newIdx = index + direction;
    if (newIdx < 0 || newIdx >= userOrderingItems.length) return;
    const updated = [...userOrderingItems];
    const temp = updated[index];
    updated[index] = updated[newIdx];
    updated[newIdx] = temp;
    userOrderingItems = updated;
  }

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
    <div class="card-front absolute inset-0 glass-card rounded-card p-6 flex flex-col items-center justify-between shadow-elevation-mid overflow-y-auto">
      {#if tags.length > 0}
        <div class="w-full flex flex-wrap gap-1 justify-center shrink-0 mb-2">
          {#each tags as tag}
            <span class="inline-flex items-center bg-white/10 text-secondary px-2 py-0.5 rounded text-[10px] font-medium tracking-wide">#{tag}</span>
          {/each}
        </div>
      {/if}

      <!-- Front Prompt -->
      <div class="flex-1 flex items-center justify-center w-full my-auto">
        <div class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance">
          {@html renderedFront}
        </div>
      </div>

      <!-- Multiple Choice Options (Front View) -->
      {#if isMultipleChoice && mcOptions.length > 0}
        <div class="w-full space-y-2 mt-4 shrink-0">
          {#each mcOptions as opt, idx}
            {@const isSelected = selectedMcIndices.includes(idx)}
            <button
              onclick={(e) => toggleMcOption(idx, e)}
              class="w-full p-2.5 rounded-xl border text-xs font-medium text-left transition-all flex items-center justify-between {isSelected ? 'bg-accent-correct/20 border-accent-correct text-primary dark:text-primary-dark shadow-sm' : 'glass border-white/10 hover:bg-white/10 text-secondary'}"
            >
              <span>{opt.text}</span>
              <span class="w-5 h-5 rounded-md border flex items-center justify-center text-xs {isSelected ? 'bg-accent-correct border-accent-correct text-white' : 'border-white/20'}">
                {isSelected ? '✓' : ''}
              </span>
            </button>
          {/each}
        </div>
      {/if}

      <!-- Ordering Items (Front View) -->
      {#if isOrdering && userOrderingItems.length > 0}
        <div class="w-full space-y-1.5 mt-3 shrink-0">
          {#each userOrderingItems as item, idx}
            <div class="flex items-center justify-between p-2 rounded-lg glass border border-white/10 text-xs text-primary dark:text-primary-dark">
              <span>{idx + 1}. {item}</span>
              <div class="flex items-center gap-1">
                <button
                  disabled={idx === 0}
                  onclick={(e) => moveItem(idx, -1, e)}
                  class="px-2 py-0.5 rounded hover:bg-white/20 disabled:opacity-30 text-xs font-bold"
                >
                  ▲
                </button>
                <button
                  disabled={idx === userOrderingItems.length - 1}
                  onclick={(e) => moveItem(idx, 1, e)}
                  class="px-2 py-0.5 rounded hover:bg-white/20 disabled:opacity-30 text-xs font-bold"
                >
                  ▼
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Back -->
    <div class="card-back absolute inset-0 glass-card rounded-card p-6 flex flex-col items-center justify-between shadow-elevation-mid overflow-y-auto">
      {#if tags.length > 0}
        <div class="w-full flex flex-wrap gap-1 justify-center shrink-0 mb-2">
          {#each tags as tag}
            <span class="inline-flex items-center bg-white/10 text-secondary px-2 py-0.5 rounded text-[10px] font-medium tracking-wide">#{tag}</span>
          {/each}
        </div>
      {/if}

      <!-- Multiple Choice Feedback (Back View) -->
      {#if isMultipleChoice && mcOptions.length > 0}
        <div class="w-full space-y-2 my-auto">
          <p class="text-xs font-bold uppercase text-secondary tracking-wider text-center mb-3">Lösung:</p>
          {#each mcOptions as opt, idx}
            {@const isSelected = selectedMcIndices.includes(idx)}
            {@const isCorrect = opt.correct}
            {@const isSuccess = isSelected && isCorrect}
            {@const isMissed = !isSelected && isCorrect}
            {@const isWrongSelect = isSelected && !isCorrect}

            <div class="w-full p-2.5 rounded-xl border text-xs font-medium flex items-center justify-between {isSuccess ? 'bg-emerald-500/20 border-emerald-500 text-emerald-400' : isMissed ? 'bg-amber-500/20 border-amber-500 text-amber-400' : isWrongSelect ? 'bg-red-500/20 border-red-500 text-red-400' : 'glass border-white/5 opacity-50 text-secondary'}">
              <span>{opt.text}</span>
              <span class="text-xs font-bold px-2 py-0.5 rounded {isSuccess ? 'bg-emerald-500/20 text-emerald-400' : isMissed ? 'bg-amber-500/20 text-amber-400' : isWrongSelect ? 'bg-red-500/20 text-red-400' : ''}">
                {isSuccess ? 'Richtig ausgewählt ✓' : isMissed ? 'Fehlend (Wäre richtiggewesen)' : isWrongSelect ? 'Falsch ausgewählt ✗' : 'Nicht zutreffend'}
              </span>
            </div>
          {/each}
        </div>
      {:else if isOrdering && originalOrderingItems.length > 0}
        <!-- Ordering Feedback (Back View) -->
        <div class="w-full space-y-2 my-auto">
          <p class="text-xs font-bold uppercase text-secondary tracking-wider text-center mb-3">Korrekte Reihenfolge:</p>
          {#each originalOrderingItems as item, idx}
            {@const userMatch = userOrderingItems[idx] === item}
            <div class="flex items-center justify-between p-2.5 rounded-xl border text-xs font-medium {userMatch ? 'bg-emerald-500/20 border-emerald-500 text-emerald-400' : 'bg-red-500/20 border-red-500 text-red-400'}">
              <span>{idx + 1}. {item}</span>
              <span class="text-xs font-bold px-2 py-0.5 rounded {userMatch ? 'bg-emerald-500/20' : 'bg-red-500/20'}">
                {userMatch ? '✓ Richtig platziert' : `Deine Wahl: ${userOrderingItems[idx] || '-'}`}
              </span>
            </div>
          {/each}
        </div>
      {:else}
        <!-- Standard / Cloze Back Content -->
        <div class="flex-1 flex items-center justify-center w-full mt-4">
          <div class="{familyClass} {sizeClass} text-primary dark:text-primary-dark text-center text-balance w-full">
            {@html renderedBack}
          </div>
        </div>
      {/if}

      {#if renderedReasoning}
        <div class="mt-4 pt-4 border-t border-white/10 w-full text-center shrink-0">
          <span class="text-[10px] uppercase text-secondary/70 font-semibold tracking-wider">Warum?</span>
          <div class="{familyClass} text-sm mt-1 text-primary/80 dark:text-primary-dark/80 text-balance opacity-90">
            {@html renderedReasoning}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Image Zoom Modal -->
{#if zoomedImageSrc}
  <div
    class="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-6 cursor-zoom-out"
    role="button"
    tabindex="0"
    onclick={(e) => { e.stopPropagation(); zoomedImageSrc = null; }}
    onkeydown={(e) => { e.stopPropagation(); (e.key === "Escape" || e.key === " ") && (zoomedImageSrc = null); }}
  >
    <div
      class="relative max-w-5xl max-h-[90vh] flex flex-col items-center justify-center"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      tabindex="-1"
    >
      <button
        onclick={(e) => { e.stopPropagation(); zoomedImageSrc = null; }}
        class="absolute -top-12 right-0 p-2 rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
        title="Schließen"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
      <img
        src={zoomedImageSrc}
        alt="Großansicht"
        class="max-w-full max-h-[85vh] rounded-2xl shadow-2xl border border-white/20 object-contain"
      />
    </div>
  </div>
{/if}
