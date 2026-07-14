<script lang="ts">
  import { untrack } from "svelte";
  import { fade } from "svelte/transition";
  import { studyStore } from "$lib/stores/study.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import * as api from "$lib/api";
  import type { Card } from "$lib/types";
  import FlashCard from "./FlashCard.svelte";
  import ScoreButtons from "./ScoreButtons.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { deckIds = [], tags = [], customCards = [], deckName = "", practiceMode = false, onReview = () => {}, onClose = () => {} } = $props<{
    deckIds?: string[];
    tags?: string[];
    customCards?: Card[];
    deckName?: string;
    practiceMode?: boolean;
    onReview?: () => void;
    onClose?: () => void;
  }>();

  let loading = $state(true);
  let empty = $state(false);
  let ratingControlsReady = $state(false);
  let showingAnswer = $state(false);

  const s = studyStore;
  let requestedSessionKey = $derived([
    practiceMode ? "practice" : "review",
    [...deckIds].sort().join(","),
    [...tags].sort().join(","),
    customCards.map((card: Card) => card.id).sort().join(","),
  ].join(":"));
  let cardFontClass = $derived(settingsStore.fontFamilyClass(settingsStore.current.card_font_family));
  let controlTransitionClass = $derived(
    settingsStore.controlTransitionAnimationEnabled() ? "transition-colors duration-200" : "",
  );

  async function loadPracticeCards(): Promise<boolean> {
    const cardsByDeck = await Promise.all(deckIds.map((deckId: string) => api.listCards(deckId)));
    const cards = tags.length > 0
      ? cardsByDeck.flat().filter((card) => card.tags.some((tag: string) => tags.includes(tag)))
      : cardsByDeck.flat();
    return s.startPracticeSession(cards, settingsStore.current.session_limit, requestedSessionKey);
  }

  function revealAnswer() {
    if (!s.isFlipped) s.flip();
    showingAnswer = true;
  }

  function toggleCardSide() {
    if (!s.isFlipped) {
      revealAnswer();
      return;
    }
    showingAnswer = !showingAnswer;
  }

  $effect(() => {
    loading = true;
    untrack(() => {
      settingsStore.load().then(() => {
        if (s.canResumeSession(requestedSessionKey)) {
          empty = false;
          loading = false;
          return;
        }

        let startPromise: Promise<boolean>;
        if (practiceMode) {
          startPromise = customCards.length > 0
            ? s.startPracticeSession(customCards, settingsStore.current.session_limit, requestedSessionKey)
            : loadPracticeCards();
        } else if (customCards && customCards.length > 0) {
          startPromise = s.startCustomSession(customCards, requestedSessionKey);
        } else if (tags.length > 0) {
          startPromise = s.startSessionByTags(tags, settingsStore.current.session_limit, requestedSessionKey);
        } else {
          startPromise = s.startSession(deckIds, settingsStore.current.session_limit, requestedSessionKey);
        }
        
        startPromise.then((hasCards) => {
          empty = !hasCards;
          loading = false;
        });
      });
    });
  });

  function handleKeydown(e: KeyboardEvent) {
    // Undo works even after session ends
    if (e.key === "z" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      s.undo();
      return;
    }

    if (!s.sessionActive) return;

    if (e.key === " " || e.key === "Spacebar") {
      e.preventDefault();
      toggleCardSide();
    } else if (ratingControlsReady) {
      switch (e.key) {
        case "1":
          handleRate(1);
          break;
        case "2":
          handleRate(2);
          break;
        case "3":
          handleRate(3);
          break;
        case "4":
          handleRate(4);
          break;
        case "Escape":
          s.pauseSession();
          onClose();
          break;
      }
    } else if (e.key === "Escape") {
      s.pauseSession();
      onClose();
    }
  }

  function handleRate(quality: number) {
    if (ratingControlsReady && s.isFlipped && s.sessionActive) {
      ratingControlsReady = false;
      showingAnswer = false;
      onReview();
      s.rate(quality);
    }
  }

  async function continueSession() {
    loading = true;
    const hasCards = s.isPractice
      ? await loadPracticeCards()
      : customCards.length > 0
        ? await s.startCustomSession(customCards, requestedSessionKey)
      : tags.length > 0
        ? await s.startSessionByTags(tags, settingsStore.current.session_limit, requestedSessionKey)
        : await s.startSession(deckIds, settingsStore.current.session_limit, requestedSessionKey);
    empty = !hasCards;
    loading = false;
  }

  async function startPractice() {
    loading = true;
    const hasCards = await loadPracticeCards();
    empty = !hasCards;
    loading = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full overflow-y-auto">
  <!-- Top Bar -->
  <div class="flex flex-wrap items-center gap-3 p-4 pb-2 sm:p-6 sm:pb-2">
    <button
      onclick={() => {
        s.pauseSession();
        onClose();
      }}
      class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
      title="Zurück (Esc)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    <h1 class="{cardFontClass} min-w-0 flex-1 truncate text-xl font-normal text-primary dark:text-primary-dark">
      {deckName}
    </h1>
    <span class="ml-11 w-full text-xs text-secondary sm:ml-auto sm:w-auto sm:text-sm">
      {s.isPractice ? `${s.dueCards.length} in freier Übung` : `${s.dueCards.length} fällig`}
    </span>
  </div>

  <!-- Progress -->
  <div class="px-4 pb-4 sm:px-6">
    <ProgressBar current={s.completedCount} total={s.sessionSize} />
  </div>

  <!-- Card Area -->
  <div class="grid flex-1 px-4 pb-6 sm:px-6">
    {#if loading}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex flex-col items-center justify-center">
        <p class="text-secondary text-lg">Lädt...</p>
      </div>
    {:else if empty}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 text-center flex flex-col items-center justify-center">
        <div class="text-6xl mb-4 opacity-20">🎉</div>
        <h2 class="text-2xl font-bold text-primary dark:text-primary-dark mb-2">
          Alles geschafft!
        </h2>
        <p class="text-secondary mb-6">Dein Lernpensum ist erledigt. Frei üben kannst du trotzdem jederzeit.</p>
        <div class="flex flex-col gap-3 sm:flex-row">
          <button onclick={onClose} class="secondary-action px-6 py-2.5 text-sm">Zurück zur Übersicht</button>
          {#if deckIds.length > 0}
            <button onclick={startPractice} class="primary-action px-6 py-2.5 text-sm">Frei weiterlernen</button>
          {/if}
        </div>
      </div>
    {:else if !s.sessionActive && !empty}
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 text-center flex flex-col items-center justify-center">
        <div class="text-6xl mb-4 opacity-20">✅</div>
        <h2 class="text-2xl font-bold text-primary dark:text-primary-dark mb-2">
          {s.isPractice ? "Übungsrunde beendet" : "Session beendet"}
        </h2>
        <p class="text-secondary mb-6">
          {s.isPractice ? "Die freie Übung hat deinen Lernplan nicht verändert." : "Gute Arbeit! Falsch beantwortete Karten kannst du jetzt wiederholen."}
        </p>
        <div class="flex gap-3 justify-center">
          <button
            onclick={onClose}
            class="rounded-button bg-white/60 dark:bg-white/10 text-primary dark:text-primary-dark px-6 py-2.5 font-medium hover:scale-[1.02] transition-transform"
          >
            Zurück
          </button>
          <button
            onclick={continueSession}
            class="rounded-button bg-accent-correct text-white px-6 py-2.5 font-medium hover:scale-[1.02] transition-transform"
          >
            {s.isPractice ? "Nochmal frei lernen" : "Weiterlernen"}
          </button>
        </div>
      </div>
    {:else if s.currentCard}
      <!-- Active card -->
      <div in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} class="col-start-1 row-start-1 flex w-full flex-col items-center justify-center gap-6">
        <div
          class="w-full cursor-pointer"
          role="button"
          tabindex="0"
          aria-label={!s.isFlipped ? "Antwort aufdecken" : showingAnswer ? "Vorderseite zeigen" : "Antwort zeigen"}
          onclick={toggleCardSide}
          onkeydown={(event) => {
            if (event.key === " " || event.key === "Enter") {
              event.preventDefault();
              event.stopPropagation();
              toggleCardSide();
            }
          }}
        >
          {#key s.currentCard.card.id}
            <FlashCard
              front={s.currentCard.card.front}
              back={s.currentCard.card.back}
              frontLanguage={s.currentCard.card.front_language}
              backLanguage={s.currentCard.card.back_language}
              reasoning={s.currentCard.card.reasoning}
              tags={s.currentCard.card.tags}
              flipped={showingAnswer}
              cardType={s.currentCard.card.card_type}
              content={s.currentCard.card.content}
              onTurnComplete={() => {
                if (s.isFlipped) ratingControlsReady = true;
              }}
            />
          {/key}
        </div>
        <div class="flex h-44 w-full shrink-0 flex-col items-center justify-start gap-3 sm:h-28">
          <button
            onclick={(event) => {
              event.stopPropagation();
              toggleCardSide();
            }}
            class="min-w-44 px-5 py-2.5 text-sm {ratingControlsReady ? 'secondary-action' : 'primary-action'} {controlTransitionClass}"
          >
            {ratingControlsReady
              ? showingAnswer
                ? "Vorderseite ansehen"
                : "Antwort ansehen"
              : "Antwort zeigen"}
          </button>
          <ScoreButtons
            enabled={ratingControlsReady}
            animate={settingsStore.ratingButtonsAnimationEnabled()}
            smooth={settingsStore.controlTransitionAnimationEnabled()}
            onRate={handleRate}
          />
        </div>
      </div>
    {/if}
  </div>
</div>
