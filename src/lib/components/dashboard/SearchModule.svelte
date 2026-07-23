<!-- Search module: square icon tile that opens the full search view. -->
<script lang="ts">
  import { Search as SearchIcon } from "@lucide/svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { t } from "$lib/i18n";
</script>

<button
  onclick={() => navigation.openSearch()}
  class="surface-panel flex aspect-square w-full items-center justify-center text-primary transition-colors hover:border-accent-correct hover:text-accent-correct dark:text-primary-dark"
  title={t("Karten durchsuchen")}
  aria-label={t("Karten durchsuchen")}
><SearchIcon size={34} strokeWidth={1.8} /></button>
