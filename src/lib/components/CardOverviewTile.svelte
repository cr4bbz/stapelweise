<script lang="ts">
  import { Repeat2 } from "@lucide/svelte";
  import { renderMarkdown } from "$lib/markdown";
  import { languageLabel } from "$lib/languages";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n";
  import type { Card } from "$lib/types";

  let { card } = $props<{ card: Card }>();

  let displayedBack = $state(false);
  let cardAngle = $state(0);
  let cardTransition = $state("none");
  let isFlipping = $state(false);
  let dragStartX = $state<number | null>(null);
  let dragDeltaX = $state(0);
  let suppressCardClick = $state(false);
  let dragAnimationFrame: number | null = null;
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));

  const wait = (duration: number) => new Promise<void>((resolve) => setTimeout(resolve, duration));
  let transform = $derived(`translateZ(0.01px) rotateY(${cardAngle}deg) scale(${1 - Math.abs(Math.sin(cardAngle * Math.PI / 180)) * 0.1})`);

  async function flipCard(directionOverride?: 1 | -1) {
    if (isFlipping) return;
    const target = !displayedBack;
    if (!settingsStore.cardFlipAnimationEnabled()) {
      displayedBack = target;
      cardAngle = target ? 180 : 0;
      return;
    }

    const direction = directionOverride ?? (target ? 1 : -1);
    const targetAngle = target ? direction * 180 : direction > 0 ? 360 : 0;
    isFlipping = true;
    displayedBack = target;
    cardTransition = "transform 280ms cubic-bezier(0.2, 0.75, 0.2, 1)";
    cardAngle = targetAngle;
    await wait(280);
    cardTransition = "none";
    cardAngle = target ? 180 : 0;
    isFlipping = false;
  }

  function handlePointerDown(event: PointerEvent) {
    if (isFlipping || event.button !== 0) return;
    dragStartX = event.clientX;
    dragDeltaX = 0;
    cardTransition = "none";
    (event.currentTarget as HTMLDivElement).setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: PointerEvent) {
    if (dragStartX === null) return;
    dragDeltaX = Math.max(-56, Math.min(56, event.clientX - dragStartX));
    if (dragAnimationFrame !== null) return;
    dragAnimationFrame = requestAnimationFrame(() => {
      dragAnimationFrame = null;
      cardAngle = (displayedBack ? 180 : 0) + dragDeltaX / 1.55;
    });
  }

  function finishDrag(event: PointerEvent) {
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
    const cardElement = event.currentTarget as HTMLDivElement;
    if (cardElement.hasPointerCapture(event.pointerId)) cardElement.releasePointerCapture(event.pointerId);

    if (movedFarEnough) {
      suppressCardClick = true;
      void flipCard(direction);
      return;
    }

    if (wasDrag) suppressCardClick = true;
    cardTransition = "transform 160ms cubic-bezier(0, 0, 0.2, 1)";
    cardAngle = displayedBack ? 180 : 0;
  }

  function handleClick() {
    if (suppressCardClick) {
      suppressCardClick = false;
      return;
    }
    void flipCard();
  }
</script>

<div class="card-overview-tile relative aspect-[5/3] min-w-0 [perspective:1400px]">
  <div
    role="button"
    tabindex="0"
    onclick={handleClick}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={finishDrag}
    onpointercancel={finishDrag}
    onkeydown={(event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        void flipCard();
      }
    }}
    class="card-overview-flip-surface group relative h-full w-full touch-none select-none text-left [will-change:transform]"
    style:transform={transform}
    style:transition={cardTransition}
    aria-label={displayedBack ? t("Vorderseite zeigen") : t("Rückseite zeigen")}
  >
    <div class="card-overview-face glass-card flex h-full w-full flex-col overflow-hidden rounded-card p-4 text-primary dark:text-primary-dark">
      <div class="flex items-start justify-between gap-2">
        <span class="section-kicker">{t("Frage")}{card.front_language ? ` · ${languageLabel(card.front_language)}` : ""}</span>
        {#if card.card_type !== "basic"}
          <span class="rounded border border-accent-correct/30 bg-accent-correct/10 px-1.5 py-0.5 text-[10px] font-bold text-accent-correct">
            {card.card_type === "multiple_choice" ? "MC" : card.card_type === "ordering" ? t("Sequenz") : card.card_type === "cloze" ? "Cloze" : t("freeTextCard")}
          </span>
        {/if}
      </div>
      <div data-user-content class="{cardFontClass} my-auto max-h-[65%] w-full overflow-hidden text-center text-primary dark:text-primary-dark">
        {@html renderMarkdown(card.front)}
      </div>
      {#if card.tags?.length}
        <div class="flex max-h-5 flex-wrap gap-1 overflow-hidden">
          {#each card.tags.slice(0, 3) as tag}
            <span class="rounded bg-white/10 px-1.5 py-0.5 text-[10px] font-medium text-secondary">#{tag}</span>
          {/each}
          {#if card.tags.length > 3}<span class="rounded bg-white/10 px-1.5 py-0.5 text-[10px] font-medium text-secondary">+{card.tags.length - 3}</span>{/if}
        </div>
      {/if}
      <Repeat2 class="absolute bottom-3 right-3 text-secondary transition-colors group-hover:text-accent-correct" size={16} />
    </div>

    <div class="card-overview-face card-overview-face-back glass-card flex h-full w-full flex-col overflow-hidden rounded-card p-4 text-primary dark:text-primary-dark">
      <span class="section-kicker">{t("Antwort")}{card.back_language ? ` · ${languageLabel(card.back_language)}` : ""}</span>
      <div data-user-content class="{cardFontClass} my-auto max-h-[65%] w-full overflow-hidden text-center text-primary dark:text-primary-dark">
        {@html renderMarkdown(card.back)}
      </div>
      {#if card.reasoning?.trim()}
        <div class="max-h-[22%] overflow-hidden border-t border-current/10 pt-2 text-center">
          <span class="text-[10px] font-semibold uppercase tracking-wider text-secondary">{t("Warum?")}</span>
          <div data-user-content class="{cardFontClass} text-xs text-primary/80 dark:text-primary-dark/80">{@html renderMarkdown(card.reasoning)}</div>
        </div>
      {/if}
      <Repeat2 class="absolute bottom-3 right-3 text-secondary transition-colors group-hover:text-accent-correct" size={16} />
    </div>
  </div>
</div>

<style>
  .card-overview-flip-surface {
    transform-style: preserve-3d;
    -webkit-transform-style: preserve-3d;
    transform-origin: center;
  }

  .card-overview-face {
    position: absolute;
    inset: 0;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
  }

  .card-overview-face-back {
    transform: rotateY(180deg);
  }

  .card-overview-face :global(p) {
    margin: 0;
  }

  .card-overview-face :global(img) {
    max-height: 100%;
    margin-inline: auto;
  }
</style>
