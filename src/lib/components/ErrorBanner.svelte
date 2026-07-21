<script lang="ts">
  let { message, onDismiss = () => {} } = $props<{
    message: string;
    onDismiss?: () => void;
  }>();
</script>

<div class="mx-6 mb-4 p-3 rounded-lg bg-accent-incorrect/10 text-accent-incorrect text-sm flex items-center justify-between">
  <span>{message}</span>
  <button onclick={onDismiss} class="font-bold ml-2">&times;</button>
</div>
