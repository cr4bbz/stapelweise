<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade, slide } from "svelte/transition";
  import * as api from "$lib/api";
  import type { Card } from "$lib/types";
  import FlashCard from "./FlashCard.svelte";

  let {
    deckIds = [],
    tags = [],
    testName = "Test-Simulation",
    onClose = () => {},
    onStudyFailed = (_cards: Card[]) => {},
  } = $props<{
    deckIds?: string[];
    tags?: string[];
    testName?: string;
    onClose?: () => void;
    onStudyFailed?: (cards: Card[]) => void;
  }>();

  // Mode: "config" | "running" | "result"
  let mode = $state<"config" | "running" | "result">("config");

  // Config State
  let selectedCount = $state<number>(10);
  let timeLimitMinutes = $state<number>(15);
  let shuffle = $state<boolean>(true);

  // Active Test State
  let allCards = $state<Card[]>([]);
  let testCards = $state<Card[]>([]);
  let currentIndex = $state<number>(0);
  let isFlipped = $state<boolean>(false);
  let isAnswering = $state<boolean>(false);
  
  // Answers tracking: card.id -> boolean (true = correct, false = incorrect)
  let userAnswers = $state<Record<string, boolean>>({});

  // Timer State
  let timeRemaining = $state<number>(0);
  let timerInterval = $state<ReturnType<typeof setInterval> | null>(null);
  let startTime = $state<number>(0);
  let endTime = $state<number>(0);

  onMount(async () => {
    try {
      if (tags.length > 0) {
        // Fetch cards by tags
        const res = await api.getDueCardsByTags(tags, 500);
        allCards = res.map((r: any) => (Array.isArray(r) ? r[0] : r.card));
      } else if (deckIds.length > 0) {
        // Fetch cards for selected decks
        const cardsArrays = await Promise.all(deckIds.map((id: string) => api.listCards(id)));
        allCards = cardsArrays.flat();
      } else {
        allCards = [];
      }
    } catch (e) {
      console.error("Failed to load cards for test simulation", e);
      allCards = [];
    }
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
  });

  function startTest() {
    if (allCards.length === 0) return;

    let pool = [...allCards];
    if (shuffle) {
      pool.sort(() => Math.random() - 0.5);
    }

    const count = selectedCount === -1 ? pool.length : Math.min(selectedCount, pool.length);
    testCards = pool.slice(0, count);
    
    currentIndex = 0;
    userAnswers = {};
    isFlipped = false;
    isAnswering = false;
    startTime = Date.now();

    if (timeLimitMinutes > 0) {
      timeRemaining = timeLimitMinutes * 60;
      timerInterval = setInterval(() => {
        timeRemaining -= 1;
        if (timeRemaining <= 0) {
          if (timerInterval) clearInterval(timerInterval);
          finishTest();
        }
      }, 1000);
    }

    mode = "running";
  }

  function handleAnswer(correct: boolean) {
    if (isAnswering || currentIndex >= testCards.length) return;
    isAnswering = true;
    
    const currentCard = testCards[currentIndex];
    userAnswers[currentCard.id] = correct;

    setTimeout(() => {
      if (currentIndex + 1 < testCards.length) {
        currentIndex += 1;
        isFlipped = false;
      } else {
        finishTest();
      }
      isAnswering = false;
    }, 50);
  }

  function finishTest() {
    if (timerInterval) clearInterval(timerInterval);

    // Mark remaining unanswered cards as false (incorrect) for score consistency
    for (const card of testCards) {
      if (userAnswers[card.id] === undefined) {
        userAnswers[card.id] = false;
      }
    }

    endTime = Date.now();
    mode = "result";
  }

  function handleKeydown(e: KeyboardEvent) {
    if (mode !== "running") return;

    if (e.key === " " || e.key === "Spacebar") {
      e.preventDefault();
      isFlipped = !isFlipped;
    } else if (isFlipped) {
      if (e.key === "1" || e.key === "Enter") {
        e.preventDefault();
        handleAnswer(true);
      } else if (e.key === "2" || e.key === "Backspace") {
        e.preventDefault();
        handleAnswer(false);
      }
    }
  }

  // Calculated Stats for Result View
  let totalAnswered = $derived(Object.keys(userAnswers).length);
  let correctCount = $derived(Object.values(userAnswers).filter(Boolean).length);
  let incorrectCount = $derived(totalAnswered - correctCount);
  let scorePercentage = $derived(testCards.length > 0 ? Math.round((correctCount / testCards.length) * 100) : 0);
  let timeTakenSeconds = $derived(Math.max(1, Math.round((endTime - startTime) / 1000)));

  let gradeBadge = $derived(() => {
    if (scorePercentage >= 90) return { label: "Sehr gut (1.0)", color: "text-emerald-500 bg-emerald-500/10 border-emerald-500/20" };
    if (scorePercentage >= 80) return { label: "Gut (2.0)", color: "text-green-500 bg-green-500/10 border-green-500/20" };
    if (scorePercentage >= 65) return { label: "Befriedigend (3.0)", color: "text-amber-500 bg-amber-500/10 border-amber-500/20" };
    if (scorePercentage >= 50) return { label: "Ausreichend (4.0)", color: "text-orange-500 bg-orange-500/10 border-orange-500/20" };
    return { label: "Nicht bestanden (5.0)", color: "text-red-500 bg-red-500/10 border-red-500/20" };
  });

  function formatTimer(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  let failedCards = $derived(testCards.filter(c => userAnswers[c.id] === false));
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-full w-full flex flex-col bg-atmosphere p-6 overflow-hidden">
  <!-- Header -->
  <div class="flex items-center justify-between pb-4 border-b border-white/10 shrink-0">
    <div class="flex items-center gap-3">
      <button
        onclick={onClose}
        class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
        title="Schließen"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
      <div>
        <h1 class="text-xl font-bold text-primary dark:text-primary-dark">🧪 Prüfungs-Simulator</h1>
        <p class="text-xs text-secondary">{testName}</p>
      </div>
    </div>

    {#if mode === "running" && timeLimitMinutes > 0}
      <div class="flex items-center gap-2 font-mono text-sm px-3 py-1.5 rounded-lg border shadow-sm {timeRemaining < 120 ? 'bg-red-500/10 border-red-500/30 text-accent-incorrect animate-pulse' : 'glass border-white/10 text-primary dark:text-primary-dark'}">
        <span>⏱️</span>
        <span class="font-bold">{formatTimer(timeRemaining)}</span>
      </div>
    {/if}
  </div>

  <!-- Content Switcher -->
  <div class="flex-1 overflow-y-auto pt-6 min-h-0">
    {#if mode === "config"}
      <!-- Configuration View -->
      <div in:fade class="max-w-xl mx-auto glass border border-white/10 rounded-2xl p-6 shadow-elevation-mid">
        <h2 class="text-lg font-bold text-primary dark:text-primary-dark mb-1">Test-Parameter konfigurieren</h2>
        <p class="text-xs text-secondary mb-6">Simuliere eine echte Prüfungssituation. Die Ergebnisse fließen nicht in dein automatisches SRS-Intervall ein.</p>

        <div class="space-y-6">
          <!-- Question Count -->
          <div>
            <span class="block text-xs font-bold text-secondary uppercase tracking-wider mb-2">Anzahl der Fragen</span>
            <div class="grid grid-cols-4 gap-2">
              {#each [5, 10, 20, -1] as cnt}
                <button
                  onclick={() => selectedCount = cnt}
                  class="py-2 text-sm font-medium rounded-xl border transition-all {selectedCount === cnt ? 'bg-accent-correct border-accent-correct text-white' : 'glass border-white/10 hover:bg-white/10 text-primary dark:text-primary-dark'}"
                >
                  {cnt === -1 ? `Alle (${allCards.length})` : cnt}
                </button>
              {/each}
            </div>
          </div>

          <!-- Time Limit -->
          <div>
            <span class="block text-xs font-bold text-secondary uppercase tracking-wider mb-2">Zeitlimit</span>
            <div class="grid grid-cols-4 gap-2">
              {#each [[0, "Ohne"], [5, "5 Min"], [15, "15 Min"], [30, "30 Min"]] as [min, label]}
                <button
                  onclick={() => timeLimitMinutes = Number(min)}
                  class="py-2 text-sm font-medium rounded-xl border transition-all {timeLimitMinutes === min ? 'bg-accent-correct border-accent-correct text-white' : 'glass border-white/10 hover:bg-white/10 text-primary dark:text-primary-dark'}"
                >
                  {label}
                </button>
              {/each}
            </div>
          </div>

          <!-- Shuffle Toggle -->
          <div class="flex items-center justify-between p-3 glass rounded-xl border border-white/10">
            <div>
              <p class="text-sm font-medium text-primary dark:text-primary-dark">Fragen mischen</p>
              <p class="text-xs text-secondary">Reihenfolge zufällig anordnen</p>
            </div>
            <input
              type="checkbox"
              bind:checked={shuffle}
              class="w-5 h-5 accent-accent-correct cursor-pointer"
            />
          </div>
        </div>

        <div class="mt-8 flex justify-end">
          <button
            onclick={startTest}
            disabled={allCards.length === 0}
            class="w-full py-3 rounded-xl bg-accent-correct text-white font-bold text-sm hover:scale-[1.01] transition-transform shadow-elevation-low disabled:opacity-50"
          >
            🚀 Simulation starten ({allCards.length} verfügbare Karten)
          </button>
        </div>
      </div>

    {:else if mode === "running"}
      <!-- Running Test View -->
      <div in:fade class="max-w-2xl mx-auto flex flex-col h-full items-center justify-between pb-6">
        <!-- Progress Indicator -->
        <div class="w-full flex items-center justify-between mb-4">
          <span class="text-xs font-bold text-secondary uppercase tracking-wider">
            Frage {currentIndex + 1} von {testCards.length}
          </span>
          <button
            onclick={finishTest}
            class="text-xs font-medium text-secondary hover:text-accent-incorrect transition-colors"
          >
            🏁 Test vorzeitig beenden
          </button>
        </div>

        <!-- Question Flashcard -->
        {#if testCards[currentIndex]}
          <div class="w-full h-80 my-auto">
            <FlashCard
              front={testCards[currentIndex].front}
              back={testCards[currentIndex].back}
              frontLanguage={testCards[currentIndex].front_language}
              backLanguage={testCards[currentIndex].back_language}
              reasoning={testCards[currentIndex].reasoning}
              tags={testCards[currentIndex].tags}
              flipped={isFlipped}
              cardType={testCards[currentIndex].card_type}
              content={testCards[currentIndex].content}
            />
          </div>
        {/if}

        <!-- Controls -->
        <div class="w-full mt-6">
          {#if !isFlipped}
            <button
              onclick={() => isFlipped = true}
              class="w-full py-3 rounded-xl glass border border-white/20 text-primary dark:text-primary-dark font-semibold hover:bg-white/10 transition-all text-sm shadow-elevation-low"
            >
              Antwort aufdecken (Leertaste)
            </button>
          {:else}
            <div class="grid grid-cols-2 gap-4">
              <button
                onclick={() => handleAnswer(false)}
                class="py-3 rounded-xl bg-accent-incorrect text-white font-bold text-sm hover:scale-[1.02] transition-transform shadow-elevation-low"
              >
                ❌ Falsch / Nicht gewusst
              </button>
              <button
                onclick={() => handleAnswer(true)}
                class="py-3 rounded-xl bg-accent-correct text-white font-bold text-sm hover:scale-[1.02] transition-transform shadow-elevation-low"
              >
                ✅ Richtig / Gewusst
              </button>
            </div>
          {/if}
        </div>
      </div>

    {:else if mode === "result"}
      <!-- Test Result Summary View -->
      <div in:fade class="max-w-2xl mx-auto glass border border-white/10 rounded-2xl p-6 shadow-elevation-mid">
        <div class="text-center pb-6 border-b border-white/10">
          <span class="inline-block px-3 py-1 rounded-full text-xs font-bold border mb-3 {gradeBadge().color}">
            {gradeBadge().label}
          </span>
          <h2 class="text-4xl font-black text-primary dark:text-primary-dark mb-1">{scorePercentage}%</h2>
          <p class="text-xs text-secondary">Ergebnis der Prüfungssimulation</p>

          <!-- Quick Metrics Grid -->
          <div class="grid grid-cols-3 gap-4 mt-6">
            <div class="glass p-3 rounded-xl text-center">
              <span class="text-xs text-secondary block mb-0.5">Richtig</span>
              <span class="text-lg font-bold text-emerald-500">{correctCount}</span>
            </div>
            <div class="glass p-3 rounded-xl text-center">
              <span class="text-xs text-secondary block mb-0.5">Falsch</span>
              <span class="text-lg font-bold text-red-500">{incorrectCount}</span>
            </div>
            <div class="glass p-3 rounded-xl text-center">
              <span class="text-xs text-secondary block mb-0.5">Zeit</span>
              <span class="text-lg font-bold text-primary dark:text-primary-dark">{formatTimer(timeTakenSeconds)}</span>
            </div>
          </div>
        </div>

        <!-- Question Detailed Breakdown -->
        <div class="mt-6 space-y-3 max-h-60 overflow-y-auto pr-2">
          <h3 class="text-xs font-bold text-secondary uppercase tracking-wider mb-2">Detailübersicht</h3>
          {#each testCards as card, idx}
            {@const isCorrect = userAnswers[card.id]}
            <div class="flex items-center justify-between p-3 rounded-xl glass border border-white/5 text-sm">
              <div class="flex items-center gap-3">
                <span class="text-xs font-bold text-secondary">{idx + 1}.</span>
                <span class="text-primary dark:text-primary-dark line-clamp-1">{card.front}</span>
              </div>
              <span class="text-xs font-bold px-2 py-0.5 rounded {isCorrect ? 'text-emerald-500 bg-emerald-500/10' : 'text-red-500 bg-red-500/10'}">
                {isCorrect ? 'Richtig' : 'Falsch'}
              </span>
            </div>
          {/each}
        </div>

        <!-- Action Buttons -->
        <div class="mt-6 pt-4 border-t border-white/10 flex items-center justify-between gap-3">
          <button
            onclick={onClose}
            class="px-4 py-2 text-xs font-medium text-secondary hover:text-primary dark:hover:text-primary-dark"
          >
            Zurück zur Übersicht
          </button>

          {#if failedCards.length > 0}
            <button
              onclick={() => onStudyFailed(failedCards)}
              class="px-5 py-2.5 rounded-xl bg-accent-correct text-white font-bold text-xs hover:scale-[1.02] transition-transform"
            >
              {failedCards.length} falsche Karten vertiefen
            </button>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
