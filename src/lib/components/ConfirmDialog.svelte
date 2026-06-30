<script lang="ts">
  let { title = "Bist du sicher?", message = "", confirmLabel = "Löschen", danger = true, onConfirm = () => {}, onCancel = () => {} } = $props<{
    title?: string;
    message?: string;
    confirmLabel?: string;
    danger?: boolean;
    onConfirm?: () => void;
    onCancel?: () => void;
  }>();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { e.stopPropagation(); onCancel(); }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" onclick={onCancel} role="dialog">
  <div class="glass rounded-card p-6 max-w-sm mx-4 shadow-elevation-high" onclick={(e) => e.stopPropagation()}>
    <h2 class="text-lg font-bold text-primary dark:text-primary-dark mb-2">{title}</h2>
    {#if message}
      <p class="text-secondary text-sm mb-4">{message}</p>
    {/if}
    <div class="flex gap-3 justify-end">
      <button
        onclick={onCancel}
        class="rounded-button bg-white/60 dark:bg-white/10 text-primary dark:text-primary-dark px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
      >
        Abbrechen
      </button>
      <button
        onclick={onConfirm}
        class="rounded-button {danger ? 'bg-red-500 hover:bg-red-600' : 'bg-accent-correct'} text-white px-4 py-2 text-sm font-medium hover:scale-[1.02] transition-transform"
      >
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
