<!-- Continue module: jump straight back into the last studied deck. -->
<script lang="ts">
  import { deckStore } from "$lib/stores/decks.svelte";
  import { navigation } from "$lib/stores/navigation.svelte";
  import { t } from "$lib/i18n";
  import type { Deck } from "$lib/types";

  const lastDeckStorageKey = "stapelweise.learning.lastDeck.v1";

  let lastDeck = $derived.by<Deck | null>(() => {
    const decks = deckStore.decks;
    if (decks.length === 0) return null;
    const savedId = typeof localStorage === "undefined" ? "" : localStorage.getItem(lastDeckStorageKey) ?? "";
    return decks.find((deck) => deck.id === savedId) ?? decks[0];
  });
</script>

<div class="surface-panel h-full p-5">
  <p class="section-kicker mb-3">{t("Weiterlernen")}</p>
  {#if lastDeck}
    <h2 data-user-content class="font-card-serif text-xl font-normal text-primary dark:text-primary-dark">{lastDeck.name}</h2>
    <button onclick={() => navigation.studyDeck(lastDeck!)} class="primary-action mt-4 px-4 py-2 text-sm">{t("Runde fortsetzen")}</button>
  {:else}
    <p class="text-sm text-secondary">{t("Sobald du einen Stapel angelegt hast, kannst du hier direkt weitermachen.")}</p>
  {/if}
</div>
