<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";

  let { onClose = () => {} } = $props<{
    onClose?: () => void;
  }>();

  const s = settingsStore;

  $effect(() => {
    s.load();
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function themeLabel(t: string): string {
    switch (t) {
      case "auto": return "Auto";
      case "light": return "Hell";
      case "dark": return "Dunkel";
      default: return t;
    }
  }

  function thresholdLabel(v: number): string {
    switch (v) {
      case 0: return "Sehr mild";
      case 1: return "Mild";
      case 2: return "Moderat";
      case 3: return "Standard";
      case 4: return "Streng";
      case 5: return "Sehr streng";
      default: return String(v);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center gap-3 p-6 pb-4">
    <button
      onclick={onClose}
      class="p-2 rounded-lg hover:bg-white/30 dark:hover:bg-white/10 text-secondary transition-colors"
      title="Zurück (Esc)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
      </svg>
    </button>
    <h1 class="text-2xl font-bold text-primary dark:text-primary-dark">
      Einstellungen
    </h1>
  </div>

  <div class="flex-1 overflow-y-auto px-6 pb-6 space-y-8">
    <!-- Section: Erscheinungsbild -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">Erscheinungsbild</h2>
      <div class="space-y-4">
        <!-- Theme -->
        <div>
          <label class="text-sm font-medium text-primary dark:text-primary-dark">Design</label>
          <p class="text-xs text-secondary mb-2">Hell, Dunkel oder automatisch nach System.</p>
          <div class="flex gap-2">
            {#each ["auto", "light", "dark"] as t}
              <button
                onclick={() => s.save({ theme: t as "auto" | "light" | "dark" })}
                class="rounded-button px-4 py-1.5 text-sm font-medium transition-transform hover:scale-[1.02] {s.current.theme === t
                  ? 'bg-accent-correct text-white'
                  : 'bg-white/40 dark:bg-white/10 text-secondary hover:text-primary dark:hover:text-primary-dark'}"
              >
                {themeLabel(t)}
              </button>
            {/each}
          </div>
        </div>

        <!-- Font Family -->
        <div>
          <label class="text-sm font-medium text-primary dark:text-primary-dark">Karten-Schriftart</label>
          <p class="text-xs text-secondary mb-2">Schriftart für Vorder- und Rückseite der Karten.</p>
          <div class="flex gap-2">
            <button
              onclick={() => s.save({ card_font_family: "serif" })}
              class="rounded-button px-4 py-1.5 text-sm font-medium transition-transform hover:scale-[1.02] {s.current.card_font_family === 'serif'
                ? 'bg-accent-correct text-white'
                : 'bg-white/40 dark:bg-white/10 text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              Serif (Georgia)
            </button>
            <button
              onclick={() => s.save({ card_font_family: "sans" })}
              class="rounded-button px-4 py-1.5 text-sm font-medium transition-transform hover:scale-[1.02] {s.current.card_font_family === 'sans'
                ? 'bg-accent-correct text-white'
                : 'bg-white/40 dark:bg-white/10 text-secondary hover:text-primary dark:hover:text-primary-dark'}"
            >
              Sans (Inter)
            </button>
          </div>
        </div>

        <!-- Font Size -->
        <div>
          <label class="text-sm font-medium text-primary dark:text-primary-dark">Karten-Schriftgröße</label>
          <p class="text-xs text-secondary mb-2">Schriftgröße für Karteninhalte im Lernmodus.</p>
          <div class="flex gap-2">
            {#each (["small", "medium", "large"] as const) as size}
              <button
                onclick={() => s.save({ card_font_size: size })}
                class="rounded-button px-4 py-1.5 text-sm font-medium transition-transform hover:scale-[1.02] {s.current.card_font_size === size
                  ? 'bg-accent-correct text-white'
                  : 'bg-white/40 dark:bg-white/10 text-secondary hover:text-primary dark:hover:text-primary-dark'}"
              >
                {size === "small" ? "Klein" : size === "medium" ? "Mittel" : "Groß"}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <!-- Section: Lernerfahrung -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">Lernerfahrung</h2>
      <div class="space-y-5">
        <!-- Session Limit -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="text-sm font-medium text-primary dark:text-primary-dark">Karten pro Session</label>
            <span class="text-sm font-semibold text-accent-correct">{s.current.session_limit}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Maximale Anzahl fälliger Karten, die pro Lern-Durchgang gezeigt werden.</p>
          <input
            type="range"
            min="5"
            max="200"
            step="5"
            value={s.current.session_limit}
            oninput={(e) => s.save({ session_limit: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>5</span>
            <span>200</span>
          </div>
        </div>

        <!-- Pass Threshold -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="text-sm font-medium text-primary dark:text-primary-dark">SM-2 Strenge</label>
            <span class="text-sm font-semibold text-accent-correct">{thresholdLabel(s.current.sm2_pass_threshold)}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Welche Mindest-Bewertung gilt als "richtig" (bestanden)? Höher = strenger.</p>
          <input
            type="range"
            min="0"
            max="5"
            step="1"
            value={s.current.sm2_pass_threshold}
            oninput={(e) => s.save({ sm2_pass_threshold: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>0 (mild)</span>
            <span>3 (standard)</span>
            <span>5 (streng)</span>
          </div>
        </div>

        <!-- Initial Ease Factor -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label class="text-sm font-medium text-primary dark:text-primary-dark">Start-Ease-Faktor</label>
            <span class="text-sm font-semibold text-accent-correct">{s.current.sm2_initial_ef.toFixed(1)}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Startwert für die SM-2-Intervall-Berechnung. Höher = längere Intervalle.</p>
          <input
            type="range"
            min="1.3"
            max="3.0"
            step="0.1"
            value={s.current.sm2_initial_ef}
            oninput={(e) => s.save({ sm2_initial_ef: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>1.3</span>
            <span>2.5 (standard)</span>
            <span>3.0</span>
          </div>
        </div>
      </div>
    </section>
  </div>
</div>
