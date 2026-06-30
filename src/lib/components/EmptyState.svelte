<script lang="ts">
  import type { Snippet } from "svelte";

  let { title = "Noch nichts da", description = "", actionLabel = "", onAction = () => {}, icon }: { title?: string; description?: string; actionLabel?: string; onAction?: () => void; icon?: Snippet } = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 text-center">
  <div class="mb-6 text-6xl opacity-20 select-none">
    {#if icon}
      {@render icon()}
    {:else}
      📚
    {/if}
  </div>
  <h3 class="text-xl font-semibold text-secondary mb-2">{title}</h3>
  {#if description}
    <p class="text-secondary max-w-sm text-balance mb-6">{description}</p>
  {/if}
  {#if actionLabel}
    <button
      onclick={onAction}
      class="rounded-button bg-[#1E3A5F] dark:bg-[#E0E0E0] dark:text-[#1A1A2E] text-white px-6 py-2.5 font-medium hover:scale-[1.02] transition-transform"
    >
      {actionLabel}
    </button>
  {/if}
</div>
