<script lang="ts">
  let { title = "Noch nichts da", description = "", actionLabel = "", onAction = () => {}, icon }: { title?: string; description?: string; actionLabel?: string; onAction?: () => void; icon?: () => string } = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 text-center">
  <div class="mb-5 flex h-16 w-16 items-center justify-center rounded-2xl border border-[#E4E7EC] bg-white text-3xl shadow-elevation-low dark:border-[#2A303B] dark:bg-[#171B24] select-none">
    {#if icon}
      {icon()}
    {:else}
      📚
    {/if}
  </div>
  <h3 class="text-xl font-bold text-primary dark:text-primary-dark mb-2">{title}</h3>
  {#if description}
    <p class="text-secondary max-w-sm text-balance mb-6">{description}</p>
  {/if}
  {#if actionLabel}
    <button
      onclick={onAction}
      class="primary-action px-6 py-2.5 text-sm"
    >
      {actionLabel}
    </button>
  {/if}
</div>
