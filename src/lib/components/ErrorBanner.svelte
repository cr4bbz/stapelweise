<script lang="ts">
  let { message, onDismiss = () => {} } = $props<{
    message: string;
    onDismiss?: () => void;
  }>();
</script>

<div class="mx-6 mb-4 p-3 rounded-lg bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 text-sm flex items-center justify-between">
  <span>{message}</span>
  <button onclick={onDismiss} class="font-bold ml-2">&times;</button>
</div>
