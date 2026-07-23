<!-- Spacer module: decorative blank space, a divider rule, or an editable note. -->
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { SpacerVariant } from "$lib/dashboard/registry";

  let { variant = "blank", note = "", onNoteChange } = $props<{
    variant?: SpacerVariant;
    note?: string;
    onNoteChange: (note: string) => void;
  }>();
</script>

{#if variant === "blank"}
  <div class="dashboard-spacer dashboard-spacer-blank" aria-hidden="true"></div>
{:else if variant === "divider"}
  <div class="dashboard-spacer dashboard-spacer-divider" aria-hidden="true"><span></span></div>
{:else}
  <div class="dashboard-spacer dashboard-spacer-note">
    <textarea
      data-user-content
      value={note}
      oninput={(event) => onNoteChange(event.currentTarget.value)}
      placeholder={t("Notiz...")}
      aria-label={t("Notizfläche")}
      class="h-full w-full resize-none bg-transparent p-4 text-sm text-primary outline-none placeholder:text-secondary dark:text-primary-dark"
    ></textarea>
  </div>
{/if}
