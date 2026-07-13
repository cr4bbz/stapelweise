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
  <div class="px-6 pb-4">
    <div class="glass border border-white/10 dark:border-white/5 rounded-xl p-4 shadow-elevation-low">
      <div class="flex items-center justify-between mb-3">
        <span class="text-xs font-bold text-secondary uppercase tracking-wider">
          Tags ({tags.length})
        </span>
        {#if selectedTags.size > 0}
          <button
            onclick={() => {
              onStudyTags(Array.from(selectedTags));
              selectedTags = new Set();
            }}
            class="rounded-button bg-accent-correct text-white px-3 py-1 text-xs font-medium hover:scale-[1.02] transition-transform"
          >
            {selectedTags.size} Tags lernen
          </button>
        {/if}
      </div>
      
      <div class="flex flex-wrap gap-2 max-h-32 overflow-y-auto pr-2 custom-scrollbar">
        {#each tags as tag}
          <button
            onclick={() => toggleTag(tag)}
            class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium border transition-all cursor-pointer {selectedTags.has(tag) 
              ? 'bg-accent-correct border-accent-correct text-white shadow-sm' 
              : 'border-white/10 dark:border-white/5 bg-white/40 dark:bg-white/5 hover:bg-white/70 dark:hover:bg-white/10 text-primary dark:text-primary-dark'}"
          >
            #{tag}
          </button>
        {/each}
      </div>
    </div>
  </div>
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
