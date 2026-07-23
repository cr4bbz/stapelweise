<!-- Settings module: square icon tile that opens the settings panel. -->
<script lang="ts">
  import { Settings as SettingsIcon } from "@lucide/svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { t } from "$lib/i18n";
</script>

<button
  onclick={() => navigation.openSettings()}
  class="surface-panel flex aspect-square w-full items-center justify-center text-primary transition-colors hover:border-accent-correct hover:text-accent-correct dark:text-primary-dark"
  title={t("Einstellungen öffnen")}
  aria-label={t("Einstellungen öffnen")}
><SettingsIcon size={34} strokeWidth={1.8} /></button>
