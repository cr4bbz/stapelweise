<script lang="ts">
  import { tick } from "svelte";
  import { Printer, X } from "@lucide/svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { languageLabel } from "$lib/languages";
  import { t } from "$lib/i18n";
  import type { Card } from "$lib/types";

  let { cards, deckName, onClose = () => {} } = $props<{
    cards: Card[];
    deckName: string;
    onClose?: () => void;
  }>();

  let rowsPerPage = $state(5);
  let showNumbers = $state(true);

  let pages = $derived.by(() => {
    const result: Card[][] = [];
    for (let index = 0; index < cards.length; index += rowsPerPage) {
      result.push(cards.slice(index, index + rowsPerPage));
    }
    return result;
  });

  function isCloze(card: Card) {
    return card.card_type === "cloze" || card.front.includes("==") || card.front.includes("{{c1::");
  }

  function frontText(card: Card) {
    if (!isCloze(card)) return card.front;
    return card.front
      .replace(/==(.*?)==/g, "________")
      .replace(/\{\{c1::(.*?)\}\}/g, "________");
  }

  function backText(card: Card) {
    if (!isCloze(card)) return card.back;
    const resolvedFront = card.front
      .replace(/==(.*?)==/g, "$1")
      .replace(/\{\{c1::(.*?)\}\}/g, "$1");
    return card.back.trim() ? `${resolvedFront}\n\n${card.back}` : resolvedFront;
  }

  async function printSheet() {
    await tick();
    window.print();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="study-sheet-print-root fixed inset-0 z-50 flex flex-col bg-[#e9edf2] text-[#171717] dark:bg-[#0f1218]">
  <header class="no-print flex flex-wrap items-center gap-3 border-b border-[#d5dae2] bg-white px-5 py-3 dark:border-[#303744] dark:bg-[#171b24]">
    <button class="icon-button" onclick={onClose} title={t("Druckansicht schließen")}>
      <X size={19} />
    </button>
    <div class="min-w-0">
      <h1 class="truncate text-base text-primary dark:text-primary-dark">{t("Studentenlernblatt")}</h1>
      <p class="text-xs text-secondary">{deckName} &middot; {cards.length} {t("Karten")} &middot; {pages.length} {t("Seiten")}</p>
    </div>

    <div class="ml-auto flex flex-wrap items-center justify-end gap-3">
      <div class="flex overflow-hidden rounded-md border border-[#d5dae2] dark:border-[#303744]" aria-label={t("Karten pro Seite")}>
        {#each [4, 5, 6] as density}
          <button
            class="h-9 min-w-10 px-3 text-sm {rowsPerPage === density ? 'bg-[#172033] text-white dark:bg-[#f4f6fa] dark:text-[#171b24]' : 'bg-white text-secondary hover:bg-[#f3f5f8] dark:bg-[#171b24] dark:hover:bg-[#222833]'}"
            onclick={() => (rowsPerPage = density)}
            title={`${density} ${t("Kartenpaare pro Seite")}`}
          >{density}</button>
        {/each}
      </div>
      <label class="flex items-center gap-2 text-sm text-secondary">
        <input type="checkbox" bind:checked={showNumbers} />
        {t("Nummern")}
      </label>
      <button class="primary-action flex items-center gap-2 px-4 py-2 text-sm" onclick={printSheet}>
        <Printer size={17} />
        {t("Drucken")}
      </button>
    </div>
  </header>

  <main class="print-preview-shell flex-1 overflow-auto p-6">
    {#each pages as page, pageIndex}
      <section
        class="study-sheet-page mx-auto mb-6 bg-white shadow-[0_4px_22px_rgba(16,24,40,0.18)]"
        style={`--sheet-row-height: ${258 / rowsPerPage}mm`}
      >
        <div class="study-sheet-header">
          <strong>{deckName}</strong>
          <span>{t("Studentenlernblatt")}</span>
          <span>{pageIndex + 1} / {pages.length}</span>
        </div>
        <div class="study-sheet-column-labels">
          <span>{t("Vorderseite")}</span>
          <span>{t("Rückseite")}</span>
        </div>

        {#each page as card, rowIndex}
          {@const cardNumber = pageIndex * rowsPerPage + rowIndex + 1}
          <article class="study-sheet-row">
            <div class="study-sheet-face">
              {#if showNumbers}<span class="study-sheet-number">{cardNumber}</span>{/if}
              {#if card.front_language}<span class="study-sheet-language">{languageLabel(card.front_language)}</span>{/if}
              <div class="study-sheet-content">{@html renderMarkdown(frontText(card))}</div>
            </div>
            <div class="study-sheet-face study-sheet-back">
              {#if card.back_language}<span class="study-sheet-language">{languageLabel(card.back_language)}</span>{/if}
              <div class="study-sheet-content">{@html renderMarkdown(backText(card))}</div>
            </div>
          </article>
        {/each}
      </section>
    {/each}
  </main>
</div>

<style>
  .study-sheet-page {
    box-sizing: border-box;
    width: 210mm;
    min-height: 297mm;
    padding: 10mm;
    font-family: Georgia, "Times New Roman", serif;
    color: #171717;
  }

  .study-sheet-header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    height: 9mm;
    border-bottom: 0.35mm solid #171717;
    font-family: "Source Sans 3", Arial, sans-serif;
    font-size: 8pt;
  }

  .study-sheet-header strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .study-sheet-header span:nth-child(2) {
    font-weight: 700;
    text-transform: uppercase;
  }

  .study-sheet-header span:last-child {
    text-align: right;
  }

  .study-sheet-column-labels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    height: 10mm;
    border-bottom: 0.25mm solid #777;
    font-family: "Source Sans 3", Arial, sans-serif;
    font-size: 7pt;
    font-weight: 700;
    letter-spacing: 0;
    text-transform: uppercase;
  }

  .study-sheet-column-labels span {
    display: flex;
    align-items: center;
    padding: 0 5mm;
  }

  .study-sheet-column-labels span:last-child {
    border-left: 0.3mm dashed #777;
  }

  .study-sheet-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    height: var(--sheet-row-height);
    break-inside: avoid;
    border-bottom: 0.25mm dotted #888;
  }

  .study-sheet-face {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    overflow: hidden;
    padding: 5mm 6mm;
  }

  .study-sheet-back {
    border-left: 0.3mm dashed #777;
  }

  .study-sheet-number,
  .study-sheet-language {
    position: absolute;
    top: 2.5mm;
    font-family: "Source Sans 3", Arial, sans-serif;
    font-size: 6.5pt;
    color: #666;
  }

  .study-sheet-number {
    left: 2.5mm;
  }

  .study-sheet-language {
    right: 2.5mm;
  }

  .study-sheet-content {
    width: 100%;
    max-height: 100%;
    overflow: hidden;
    font-size: 10.5pt;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .study-sheet-content :global(p) {
    margin: 0.2em 0;
  }

  .study-sheet-content :global(ul),
  .study-sheet-content :global(ol) {
    margin: 0.25em 0;
    padding-left: 1.2em;
  }

  .study-sheet-content :global(img) {
    max-width: 100%;
    max-height: 26mm;
    margin: 1mm auto;
    object-fit: contain;
    box-shadow: none;
  }

  @media print {
    @page {
      size: A4 portrait;
      margin: 0;
    }

    :global(html),
    :global(body) {
      width: 210mm;
      height: auto;
      overflow: visible !important;
      background: white !important;
    }

    :global(body *) {
      visibility: hidden !important;
    }

    .study-sheet-print-root,
    :global(.study-sheet-print-root *) {
      visibility: visible !important;
    }

    .study-sheet-print-root {
      position: absolute;
      inset: 0;
      display: block;
      width: 210mm;
      height: auto;
      background: white;
    }

    .no-print {
      display: none !important;
    }

    .print-preview-shell {
      display: block;
      overflow: visible;
      padding: 0;
    }

    .study-sheet-page {
      margin: 0;
      box-shadow: none;
      break-after: page;
      page-break-after: always;
    }

    .study-sheet-page:last-child {
      break-after: auto;
      page-break-after: auto;
    }
  }
</style>
