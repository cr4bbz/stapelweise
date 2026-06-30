<script lang="ts">
  let { front = "", back = "", flipped = false } = $props();
</script>

<div class="card-flip w-full max-w-2xl mx-auto h-80">
  <div class="card-flip-inner relative w-full h-full {flipped ? 'flipped' : ''}">
    <!-- Front -->
    <div class="card-front absolute inset-0 glass rounded-card p-8 flex items-center justify-center">
      <p class="font-serif text-2xl text-primary dark:text-primary-dark text-center text-balance select-none">
        {front}
      </p>
    </div>
    <!-- Back -->
    <div class="card-back absolute inset-0 glass rounded-card p-8 flex items-center justify-center">
      <p class="font-serif text-2xl text-primary dark:text-primary-dark text-center text-balance select-none">
        {back}
      </p>
    </div>
  </div>
</div>
