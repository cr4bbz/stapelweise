<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "$lib/api";

  let { onStudyTags = (_tags: string[]) => {} } = $props<{
    onStudyTags: (tags: string[]) => void;
  }>();

  let tags = $state<string[]>([]);
  let selectedTags = $state<Set<string>>(new Set());

  onMount(async () => {
    try {
      tags = await api.getAllTags();
    } catch {
      tags = [];
    }
  });

  function toggleTag(tag: string) {
    const next = new Set(selectedTags);
    if (next.has(tag)) {
      next.delete(tag);
    } else {
      next.add(tag);
    }
    selectedTags = next;
  }
</script>

{#if tags.length > 0}
  <div>
    <div class="surface-panel p-4">
      <div class="flex items-center justify-between mb-3">
        <span class="section-kicker">
          Tags ({tags.length})
        </span>
        {#if selectedTags.size > 0}
          <button
            onclick={() => {
              onStudyTags(Array.from(selectedTags));
              selectedTags = new Set();
            }}
            class="primary-action px-3 py-1 text-xs"
          >
            {selectedTags.size} Tags lernen
          </button>
        {/if}
      </div>
      
      <div class="flex flex-wrap gap-2 max-h-32 overflow-y-auto pr-2 custom-scrollbar">
        {#each tags as tag}
          <button
            onclick={() => toggleTag(tag)}
            class="inline-flex items-center rounded-lg px-3 py-1 text-xs font-medium border transition-all cursor-pointer {selectedTags.has(tag) 
              ? 'bg-accent-correct border-accent-correct text-white shadow-sm' 
              : 'border-[#E4E7EC] dark:border-[#2A303B] bg-white dark:bg-[#171B24] hover:border-accent-correct/40 text-primary dark:text-primary-dark'}"
          >
            #{tag}
          </button>
        {/each}
      </div>
    </div>
  </div>
{:else}
  <section class="surface-panel p-4">
    <div class="mb-3 flex items-center justify-between">
      <h2 class="section-heading">Tags</h2>
      <span class="section-kicker">0</span>
    </div>
    <p class="text-sm text-secondary">Tags erscheinen hier, sobald du sie einer Karte zuweist.</p>
  </section>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(150, 150, 150, 0.3);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(150, 150, 150, 0.5);
  }
</style>
