<script lang="ts">
  import { ChevronDown, ListFilter, Repeat2, X } from "@lucide/svelte";
  import { onMount, tick } from "svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { languageLabel } from "$lib/languages";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
  import type { Card, Deck } from "$lib/types";

  let {
    decks = [],
    cards = [],
    selectedCardId = "",
    compact = false,
    onSelect = (_cardId: string) => {},
  } = $props<{
    decks?: Deck[];
    cards?: Card[];
    selectedCardId?: string;
    compact?: boolean;
    onSelect?: (cardId: string) => void;
  }>();

  let selectedDeckId = $state("");
  let displayedBack = $state(false);
  let cardAngle = $state(0);
  let cardTransition = $state("none");
  let isFlipping = $state(false);
  let pickerAngleX = $state(0);
  let pickerTransition = $state("none");
  let isPickerActive = $state(false);
  let isPickerFlipping = $state(false);
  let pickerHadCard = $state(false);
  let dragStartX = $state<number | null>(null);
  let dragDeltaX = $state(0);
  let suppressCardClick = $state(false);
  let dragAnimationFrame: number | null = null;
  let showCardPicker = $state(false);
  let showReasoningDetail = $state(false);
  let reasoningFitsInline = $state(true);
  let backTextScale = $state(1);
  let moduleElement = $state<HTMLDivElement | null>(null);
  let cardBodyElement = $state<HTMLDivElement | null>(null);
  let answerElement = $state<HTMLDivElement | null>(null);
  let reasoningElement = $state<HTMLDivElement | null>(null);
  let reasoningMeasurementElement = $state<HTMLDivElement | null>(null);
  let previousCardId = "";
  let selectedCard = $derived(cards.find((card: Card) => card.id === selectedCardId) ?? null);
  let deckCards = $derived(cards.filter((card: Card) => card.deck_id === selectedDeckId));
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));
  let renderedReasoning = $derived(selectedCard?.reasoning?.trim() ? renderMarkdown(selectedCard.reasoning) : null);

  $effect(() => {
    const selectedDeck = selectedCard?.deck_id;
    if (selectedDeck) selectedDeckId = selectedDeck;
    else if (!decks.some((deck: Deck) => deck.id === selectedDeckId)) selectedDeckId = decks[0]?.id ?? "";
  });

  $effect(() => {
    if (selectedCardId !== previousCardId) {
      previousCardId = selectedCardId;
      displayedBack = false;
      cardAngle = 0;
      cardTransition = "none";
      isFlipping = false;
      pickerAngleX = 0;
      pickerTransition = "none";
      isPickerActive = false;
      isPickerFlipping = false;
      pickerHadCard = false;
      showReasoningDetail = false;
      backTextScale = 1;
    }
  });

  let inlineReasoningVisible = $derived(
    displayedBack && !!renderedReasoning && (!compact || reasoningFitsInline)
  );
  let cardFaceTransform = $derived(
    (isPickerActive || isPickerFlipping)
      ? `translateZ(0.01px) rotateX(${pickerAngleX}deg)`
      : `translateZ(0.01px) rotateY(${cardAngle}deg) scale(${1 - Math.abs(Math.sin(cardAngle * Math.PI / 180)) * 0.1})`
  );
  let cardFaceTransition = $derived((isPickerActive || isPickerFlipping) ? pickerTransition : cardTransition);

  function updateReasoningLayout() {
    if (!compact || !displayedBack || !renderedReasoning) {
      reasoningFitsInline = true;
      showReasoningDetail = false;
      return;
    }

    const bodyStyle = cardBodyElement ? getComputedStyle(cardBodyElement) : null;
    const contentHeight = (cardBodyElement?.clientHeight ?? 0)
      - Number.parseFloat(bodyStyle?.paddingTop ?? "0")
      - Number.parseFloat(bodyStyle?.paddingBottom ?? "0");
    const availableHeight = contentHeight - Math.min(answerElement?.scrollHeight ?? 0, contentHeight) - 12;
    reasoningFitsInline = (reasoningMeasurementElement?.scrollHeight ?? Number.POSITIVE_INFINITY) <= availableHeight + 1;
    if (reasoningFitsInline) showReasoningDetail = false;

    const reasoningHeight = inlineReasoningVisible ? (reasoningElement?.scrollHeight ?? 0) + 12 : 0;
    const requiredHeight = (answerElement?.scrollHeight ?? 0) + reasoningHeight;
    if (requiredHeight > contentHeight + 1) {
      backTextScale = Math.max(0.45, backTextScale * (contentHeight / requiredHeight));
    }
  }

  $effect(() => {
    void selectedCardId;
    void displayedBack;
    void renderedReasoning;
    void compact;
    void reasoningFitsInline;
    void backTextScale;
    const frame = requestAnimationFrame(updateReasoningLayout);
    return () => cancelAnimationFrame(frame);
  });

  onMount(() => {
    if (!moduleElement || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(updateReasoningLayout);
    observer.observe(moduleElement);
    return () => observer.disconnect();
  });

  const wait = (duration: number) => new Promise<void>((resolve) => setTimeout(resolve, duration));

  async function flipCard(directionOverride?: 1 | -1) {
    if (isFlipping || showCardPicker || isPickerFlipping) return;
    const target = !displayedBack;
    if (!settingsStore.cardFlipAnimationEnabled()) {
      displayedBack = target;
      cardAngle = target ? 180 : 0;
      if (!target) {
        showReasoningDetail = false;
        backTextScale = 1;
      }
      return;
    }

    const direction = directionOverride ?? (target ? 1 : -1);
    const targetAngle = target
      ? direction * 180
      : direction > 0 ? 360 : 0;
    isFlipping = true;
    // Both parts of the back need to exist before the 3D turn starts.
    // Otherwise the explanation is inserted only after the answer is visible.
    displayedBack = target;
    if (!target) {
      showReasoningDetail = false;
      backTextScale = 1;
    }
    cardTransition = "transform 280ms cubic-bezier(0.2, 0.75, 0.2, 1)";
    cardAngle = targetAngle;
    await wait(280);
    cardTransition = "none";
    cardAngle = target ? 180 : 0;
    isFlipping = false;
  }

  function handleCardPointerDown(event: PointerEvent) {
    if (isFlipping || showCardPicker || isPickerFlipping || event.button !== 0) return;
    dragStartX = event.clientX;
    dragDeltaX = 0;
    cardTransition = "none";
    const cardElement = event.currentTarget as HTMLDivElement | null;
    cardElement?.setPointerCapture(event.pointerId);
  }

  function handleCardPointerMove(event: PointerEvent) {
    if (dragStartX === null) return;
    dragDeltaX = Math.max(-56, Math.min(56, event.clientX - dragStartX));
    if (dragAnimationFrame !== null) return;
    dragAnimationFrame = requestAnimationFrame(() => {
      dragAnimationFrame = null;
      cardAngle = (displayedBack ? 180 : 0) + dragDeltaX / 1.55;
    });
  }

  function finishCardDrag(event: PointerEvent) {
    if (dragStartX === null) return;
    dragDeltaX = Math.max(-56, Math.min(56, event.clientX - dragStartX));
    if (dragAnimationFrame !== null) {
      cancelAnimationFrame(dragAnimationFrame);
      dragAnimationFrame = null;
    }
    cardAngle = (displayedBack ? 180 : 0) + dragDeltaX / 1.55;
    const movedFarEnough = Math.abs(dragDeltaX) >= 46;
    const wasDrag = Math.abs(dragDeltaX) > 4;
    const direction = dragDeltaX >= 0 ? 1 : -1;
    dragStartX = null;
    const cardElement = event.currentTarget as HTMLDivElement | null;
    if (cardElement?.hasPointerCapture(event.pointerId)) cardElement.releasePointerCapture(event.pointerId);

    if (movedFarEnough) {
      suppressCardClick = true;
      void flipCard(direction);
      return;
    }

    if (wasDrag) suppressCardClick = true;
    cardTransition = "transform 160ms cubic-bezier(0, 0, 0.2, 1)";
    cardAngle = displayedBack ? 180 : 0;
  }

  function handleCardClick() {
    if (suppressCardClick) {
      suppressCardClick = false;
      return;
    }
    void flipCard();
  }

  function stopCardDrag(event: PointerEvent) {
    event.stopPropagation();
  }

  async function openCardPicker() {
    if (isFlipping || isPickerFlipping) return;
    if (isPickerActive) {
      await closeCardPicker();
      return;
    }
    pickerHadCard = !!selectedCard;
    if (!selectedCard) {
      showCardPicker = true;
      isPickerActive = true;
      pickerAngleX = -180;
      return;
    }

    displayedBack = false;
    cardAngle = 0;

    showCardPicker = true;
    isPickerFlipping = true;
    pickerTransition = "transform 320ms cubic-bezier(0.2, 0.8, 0.2, 1)";
    pickerAngleX = -180;
    await wait(320);

    pickerTransition = "none";
    isPickerActive = true;
    isPickerFlipping = false;
  }

  async function closeCardPicker() {
    if (isPickerFlipping || (!isPickerActive && !isPickerFlipping)) return;
    if (!pickerHadCard) {
      showCardPicker = false;
      isPickerActive = false;
      pickerAngleX = 0;
      return;
    }

    isPickerFlipping = true;
    pickerTransition = "transform 320ms cubic-bezier(0.2, 0.8, 0.2, 1)";
    pickerAngleX = 0;
    await wait(320);

    pickerTransition = "none";
    showCardPicker = false;
    isPickerActive = false;
    isPickerFlipping = false;
    pickerHadCard = false;
  }

  function selectDeck(deckId: string) {
    selectedDeckId = deckId;
    if (!cards.some((card: Card) => card.id === selectedCardId && card.deck_id === deckId)) onSelect("");
  }

  function selectCard(cardId: string) {
    onSelect(cardId);
    if (cardId) void closeCardPicker();
  }
</script>

<div bind:this={moduleElement} class="dashboard-single-card surface-panel relative flex h-full p-4 sm:p-5" style="perspective: 1400px">
  {#if selectedCard}
    <div
      role="button"
      tabindex="0"
      onclick={handleCardClick}
      onpointerdown={handleCardPointerDown}
      onpointermove={handleCardPointerMove}
      onpointerup={finishCardDrag}
      onpointercancel={finishCardDrag}
      onkeydown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          void flipCard();
        }
      }}
      class="single-card-flip-surface surface-panel module-accent-subpanel group relative h-full w-full touch-none select-none rounded-xl text-center text-primary transition-colors group-hover:border-accent-correct/45 [will-change:transform] dark:text-primary-dark"
      style:transform={cardFaceTransform}
      style:transition={cardFaceTransition}
      aria-label={displayedBack ? t("Vorderseite zeigen") : t("Rückseite zeigen")}
    >
      <div class="single-card-face flex flex-col items-center justify-center overflow-hidden rounded-[inherit] bg-transparent p-4 text-primary dark:text-primary-dark sm:p-6">
        <span class="absolute left-3 top-3 text-[10px] font-semibold uppercase text-secondary">
          {t("Vorderseite")}
          {selectedCard.front_language ? ` · ${languageLabel(selectedCard.front_language)}` : ""}
        </span>
        <span class="absolute bottom-3 right-3 text-secondary transition-colors group-hover:text-accent-correct"><Repeat2 size={18} /></span>
        <div data-user-content class="single-card-main prose prose-sm max-h-full max-w-full overflow-hidden px-1 {cardFontClass}">
          {@html renderMarkdown(selectedCard.front)}
        </div>
        <button
          onpointerdown={stopCardDrag}
          onclick={(event) => {
            event.stopPropagation();
            void openCardPicker();
          }}
          class="icon-button absolute right-3 top-3 z-10 !h-8 !w-8"
          aria-label={t("Karte auswählen")}
          title={t("Karte auswählen")}
        >
          <ListFilter size={16} />
        </button>
      </div>

      <div class="single-card-face single-card-face-back flex flex-col items-center justify-center overflow-hidden rounded-[inherit] bg-transparent p-4 text-primary dark:text-primary-dark sm:p-6">
        <span class="absolute left-3 top-3 text-[10px] font-semibold uppercase text-secondary">
          {t("Rückseite")}
          {selectedCard.back_language ? ` · ${languageLabel(selectedCard.back_language)}` : ""}
        </span>
        <span class="absolute bottom-3 right-3 text-secondary transition-colors group-hover:text-accent-correct"><Repeat2 size={18} /></span>
        <div bind:this={cardBodyElement} class="single-card-body flex h-full w-full min-h-0 flex-col items-center justify-center overflow-hidden pb-5 pt-5">
          <div
            bind:this={answerElement}
            data-user-content
            class:single-card-main-scroll={inlineReasoningVisible}
            class="single-card-main prose prose-sm max-h-full max-w-full overflow-hidden px-1 {cardFontClass}"
            style:font-size={compact ? `${0.75 * backTextScale}rem` : undefined}
          >
            {@html renderMarkdown(selectedCard.back)}
          </div>
          {#if inlineReasoningVisible}
            <div bind:this={reasoningElement} class="single-card-reasoning mt-3 w-full overflow-hidden border-t border-current/10 pt-3 text-center" style:font-size={compact ? `${0.6875 * backTextScale}rem` : undefined}>
              <span class="text-[10px] font-semibold uppercase tracking-wider text-secondary/80">{t("Warum?")}</span>
              <div data-user-content class="prose prose-xs mt-1 max-w-full text-primary/80 dark:text-primary-dark/80 {cardFontClass}">
                {@html renderedReasoning}
              </div>
            </div>
          {/if}
        </div>
        {#if compact && renderedReasoning && !reasoningFitsInline}
          <button
            onpointerdown={stopCardDrag}
            onclick={(event) => {
              event.stopPropagation();
              showReasoningDetail = true;
            }}
            class="why-action absolute bottom-3 left-3 z-10"
          >
            {t("Warum?")}
          </button>
        {/if}
        <button
          onpointerdown={stopCardDrag}
          onclick={(event) => {
            event.stopPropagation();
            void openCardPicker();
          }}
          class="icon-button absolute right-3 top-3 z-10 !h-8 !w-8"
          aria-label={t("Karte auswählen")}
          title={t("Karte auswählen")}
        >
          <ListFilter size={16} />
        </button>
      </div>

      {#if showCardPicker}
        <div
          class:card-picker-compact={compact}
          class:pointer-events-none={!isPickerActive && !isPickerFlipping}
          class="single-card-face single-card-face-picker card-picker-panel module-accent-subpanel z-20 flex flex-col rounded-xl p-3.5 shadow-lg sm:p-4"
          role="dialog"
          tabindex="-1"
          aria-label={t("Karte auswählen")}
          onpointerdown={stopCardDrag}
          onclick={(event) => event.stopPropagation()}
          onkeydown={(event) => event.stopPropagation()}
        >
          <div class="mb-3 text-center">
            <p class="section-kicker">{t("Karte auswählen")}</p>
            <button onclick={() => void closeCardPicker()} class="icon-button absolute right-3 top-3 !h-8 !w-8" aria-label={t("Schließen")}>
              <X size={16} />
            </button>
          </div>
          <div class="card-picker-fields space-y-3">
            <label class="card-picker-field block space-y-1.5">
              <span class="text-[10px] font-semibold uppercase tracking-wider text-secondary">{t("Stapel")}</span>
              <div class="card-picker-select-wrap">
                <select
                  value={selectedDeckId}
                  onchange={(event) => selectDeck(event.currentTarget.value)}
                  aria-label={t("Stapel auswählen")}
                  class="module-accent-input block w-full min-w-0 appearance-none truncate rounded-md py-2 pl-3 pr-10 text-sm outline-none"
                >
                  {#each decks as deck}<option value={deck.id}>{deck.name}</option>{/each}
                </select>
                <ChevronDown class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-secondary" size={15} />
              </div>
            </label>
            <label class="card-picker-field block space-y-1.5">
              <span class="text-[10px] font-semibold uppercase tracking-wider text-secondary">{t("Karte")}</span>
              <div class="card-picker-select-wrap">
                <select
                  value={selectedCardId}
                  onchange={(event) => selectCard(event.currentTarget.value)}
                  aria-label={t("Karte auswählen")}
                  class="module-accent-input block w-full min-w-0 appearance-none truncate rounded-md py-2 pl-3 pr-10 text-sm outline-none"
                >
                  <option value="">{t("Karte auswählen")}</option>
                  {#each deckCards as card, index}
                    <option value={card.id}>{index + 1}. {card.front.replace(/[#*_`]/g, "").slice(0, 48)}</option>
                  {/each}
                </select>
                <ChevronDown class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-secondary" size={15} />
              </div>
            </label>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="module-accent-subpanel flex h-full w-full flex-col items-center justify-center rounded-lg border-dashed px-4 text-center">
      <p class="text-sm font-semibold text-primary dark:text-primary-dark">{t("Karte auswählen")}</p>
      <button onclick={openCardPicker} class="secondary-action-button mt-3 text-xs">
        {t("Aus Stapel wählen")}
      </button>
    </div>
  {/if}

  {#if displayedBack && renderedReasoning}
    <div bind:this={reasoningMeasurementElement} class="single-card-reasoning-measurement" aria-hidden="true">
      <span class="text-[10px] font-semibold uppercase tracking-wider text-secondary/80">{t("Warum?")}</span>
      <div data-user-content class="prose prose-xs mt-1 max-w-full text-primary/80 dark:text-primary-dark/80 {cardFontClass}">
        {@html renderedReasoning}
      </div>
    </div>
  {/if}

  {#if showReasoningDetail && renderedReasoning}
    <div
      onclick={() => (showReasoningDetail = false)}
      onkeydown={(event) => {
        if (event.key === "Enter" || event.key === " " || event.key === "Escape") {
          event.preventDefault();
          showReasoningDetail = false;
        }
      }}
      role="button"
      tabindex="0"
      aria-label={t("Schließen")}
      class="why-detail-overlay module-accent-subpanel absolute inset-4 z-30 flex cursor-pointer min-h-0 flex-col rounded-xl p-3.5 shadow-lg sm:inset-5 sm:p-4"
    >
      <div class="mb-3 flex items-center justify-between gap-3">
        <p class="section-kicker">{t("Warum?")}</p>
        <button onclick={() => (showReasoningDetail = false)} class="icon-button !h-8 !w-8" aria-label={t("Schließen")}>
          <X size={16} />
        </button>
      </div>
      <div data-user-content class="prose prose-sm min-h-0 max-w-full flex-1 overflow-y-auto pr-1 {cardFontClass}">
        {@html renderedReasoning}
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard-single-card {
    container-type: inline-size;
    /* Backdrop filters flatten child 3D contexts in Chromium. */
    backdrop-filter: none !important;
    -webkit-backdrop-filter: none !important;
  }

  .single-card-flip-surface {
    transform-style: preserve-3d;
    -webkit-transform-style: preserve-3d;
    transform-origin: center;
    backdrop-filter: none !important;
    -webkit-backdrop-filter: none !important;
  }

  .single-card-face {
    position: absolute;
    inset: 0;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    transform: none;
  }

  .single-card-face-back {
    transform: rotateY(180deg);
  }

  .single-card-face-picker {
    transform: rotateX(180deg);
  }

  .single-card-main-scroll {
    flex: 0 0 auto;
    min-height: 0;
    width: 100%;
    overflow: hidden;
  }

  .single-card-reasoning {
    flex: 0 1 auto;
    min-height: 0;
  }

  .single-card-reasoning-measurement {
    position: absolute;
    top: 0;
    left: 1rem;
    width: calc(100% - 2rem);
    visibility: hidden;
    pointer-events: none;
    border-top: 1px solid currentColor;
    padding-top: 0.75rem;
  }

  .card-picker-select-wrap {
    position: relative;
  }

  .card-picker-select-wrap::after {
    position: absolute;
    top: 1px;
    right: 2rem;
    bottom: 1px;
    width: 2.5rem;
    content: "";
    pointer-events: none;
    background: linear-gradient(to right, transparent, rgb(255 255 255 / 0.5));
  }

  .why-action {
    display: inline-flex;
    box-sizing: border-box;
    height: 1.5rem;
    align-items: center;
    justify-content: center;
    border: 1px solid color-mix(in srgb, var(--color-line) 78%, rgb(var(--dashboard-module-accent)) 22%);
    border-radius: 0.375rem;
    background: color-mix(in srgb, var(--color-surface) 96%, rgb(var(--dashboard-module-accent)) 4%);
    padding: 0 0.625rem;
    color: rgb(var(--dashboard-module-accent));
    font-size: 0.625rem;
    font-weight: 650;
    line-height: 1;
    white-space: nowrap !important;
    overflow-wrap: normal !important;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .why-action:hover {
    border-color: rgb(var(--dashboard-module-accent) / 0.4);
    background: rgb(var(--dashboard-module-accent) / 0.08);
  }

  .card-picker-compact {
    padding: 0.75rem;
  }

  .card-picker-compact .card-picker-fields {
    display: grid;
    gap: 0.5rem;
  }

  .card-picker-compact .card-picker-fields > * + * {
    margin-top: 0 !important;
  }

  .card-picker-compact .card-picker-field {
    display: grid;
    grid-template-columns: 3.25rem minmax(0, 1fr);
    align-items: center;
    gap: 0.5rem;
  }

  .card-picker-compact .card-picker-field > span {
    white-space: nowrap;
  }

  .single-card-main {
    font-size: clamp(0.78125rem, 3.6cqw + 0.22rem, 1.25rem);
    line-height: 1.35;
  }

  .single-card-main :global(p),
  .single-card-main :global(li) {
    font-size: inherit;
    line-height: inherit;
  }

  .single-card-reasoning {
    font-size: clamp(0.6875rem, 2.8cqw + 0.18rem, 1.05rem);
    line-height: 1.3;
  }

  .single-card-reasoning :global(p),
  .single-card-reasoning :global(li),
  .single-card-reasoning-measurement :global(p),
  .single-card-reasoning-measurement :global(li) {
    font-size: inherit;
    line-height: inherit;
  }

  @container (max-width: 18rem) {
    .single-card-body {
      padding-top: 1.4rem;
      padding-bottom: 1.4rem;
    }

    .single-card-main {
      font-size: clamp(0.72rem, 3.2cqw + 0.2rem, 0.95rem);
      line-height: 1.28;
    }

    .single-card-main :global(h1),
    .single-card-main :global(h2),
    .single-card-main :global(h3) {
      font-size: 0.9rem;
      line-height: 1.15;
    }

    .single-card-reasoning {
      margin-top: 0.5rem;
      padding-top: 0.5rem;
      font-size: clamp(0.65rem, 2.5cqw + 0.15rem, 0.85rem);
      line-height: 1.25;
    }

    .single-card-reasoning-measurement {
      margin-top: 0.5rem;
      padding-top: 0.5rem;
      font-size: clamp(0.65rem, 2.5cqw + 0.15rem, 0.85rem);
      line-height: 1.25;
    }
  }
</style>
