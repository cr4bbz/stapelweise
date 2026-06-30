<script lang="ts">
  import { untrack } from "svelte";
  import { studyStore } from "$lib/stores/study.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import FlashCard from "./FlashCard.svelte";
  import ScoreButtons from "./ScoreButtons.svelte";
  import ProgressBar from "./ProgressBar.svelte";

  let { deckId, deckName = "", onClose = () => {} } = $props<{
    deckId: string;
    deckName?: string;
    onClose?: () => void;
  }>();

  let loading = $state(true);
  let empty = $state(false);

  const s = studyStore;

  $effect(() => {
    loading = true;
    untrack(() => {
      settingsStore.load().then(() => {
        s.startSession(deckId, settingsStore.current.session_limit).then((hasCards) => {
          empty = !hasCards;
          loading = false;
        });
      });
    });
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!s.sessionActive) return;

    if (e.key === " " || e.key === "Spacebar") {
      e.preventDefault();
      if (!s.isFlipped) {
        s.flip();
      }
    } else if (s.isFlipped) {
      switch (e.key) {
        case "1":
          s.rate(1);
          break;
        case "2":
          s.rate(2);
          break;
        case "3":
          s.rate(3);
          break;
        case "4":
          s.rate(4);
          break;
        case "Escape":
          s.endSession();
          onClose();
          break;
      }
    } else if (e.key === "Escape") {
      s.endSession();
      onClose();
    }
  }

  function handleRate(quality: number) {
    if (s.isFlipped && s.sessionActive) {
      s.rate(quality);
    }
  }

  async function continueSession() {
    loading = true;
    const hasCards = await s.startSession(deckId, settingsStore.current.session_limit);
    empty = !hasCards;
    loading = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Top Bar -->
  <div class="flex items-center gap-3 p-6 pb-2">
    <button
      onclick={() => {
        s.endSession();
        onClose();
      }}
      class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
      title="Zurück (Esc)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    <h1 class="text-xl font-bold text-primary dark:text-primary-dark truncate">
      {deckName}
    </h1>
    <span class="text-secondary text-sm ml-auto">{s.dueCards.length} fällig</span>
  </div>

  <!-- Progress -->
  <div class="px-6 pb-4">
    <ProgressBar current={s.currentIndex} total={s.dueCards.length} />
  </div>

  <!-- Card Area -->
  <div class="flex-1 flex flex-col items-center justify-center px-6 pb-6">
    {#if loading}
      <p class="text-secondary text-lg">Lädt...</p>
    {:else if empty}
      <div class="text-center">
        <div class="text-6xl mb-4 opacity-20">🎉</div>
        <h2 class="text-2xl font-bold text-primary dark:text-primary-dark mb-2">
          Alles geschafft!
        </h2>
        <p class="text-secondary mb-6">Keine fälligen Karten in diesem Stapel.</p>
        <button
          onclick={onClose}
          class="rounded-button bg-accent-correct text-white px-6 py-2.5 font-medium hover:scale-[1.02] transition-transform"
        >
          Zurück zur Übersicht
        </button>
      </div>
    {:else if !s.sessionActive && !empty}
      <div class="text-center">
        <div class="text-6xl mb-4 opacity-20">✅</div>
        <h2 class="text-2xl font-bold text-primary dark:text-primary-dark mb-2">
          Session beendet
        </h2>
        <p class="text-secondary mb-6">Gute Arbeit! Falsch beantwortete Karten kannst du jetzt wiederholen.</p>
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
            Weiterlernen
          </button>
        </div>
      </div>
    {:else if s.currentCard}
      <!-- Active card -->
      <div class="flex flex-col items-center gap-6 w-full" role="button" tabindex="0" onclick={() => !s.isFlipped && s.flip()} onkeydown={(e) => e.key === " " && !s.isFlipped && s.flip()}>
        <FlashCard
          front={s.currentCard.card.front}
          back={s.currentCard.card.back}
          flipped={s.isFlipped}
        />
        <ScoreButtons visible={s.isFlipped} onRate={handleRate} />
      </div>
    {/if}
  </div>
</div>
