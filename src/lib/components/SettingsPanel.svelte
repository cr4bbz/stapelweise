<script lang="ts">
  import * as api from "$lib/api";
  import { deckStore } from "$lib/stores/decks.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { colorSuggestions, colorThemes, type ColorTheme } from "$lib/themes";
  import type { AppSettings, ModuleColorSlot, ModuleColorTarget } from "$lib/types";
  import { t, uiLanguageOptions, type UiLanguage } from "$lib/i18n";
  import IntegrationImports from "./IntegrationImports.svelte";

  let { onClose = () => {} } = $props<{
    onClose?: () => void;
  }>();

  const s = settingsStore;
  type AnimationSetting = "card_flip_animation" | "control_transition_animation" | "rating_buttons_animation";
  const animationOptions: { key: AnimationSetting; label: string }[] = [
    { key: "card_flip_animation", label: "Karten wenden" },
    { key: "control_transition_animation", label: "Bedienelemente wechseln" },
    { key: "rating_buttons_animation", label: "Bewertungstasten einblenden" },
  ];
  const moduleColorTargets: { id: ModuleColorTarget; label: string }[] = [
    { id: "deck", label: "Karteikartenstapel" },
    { id: "single_card", label: "Einzelkarte" },
    { id: "timer", label: "Lerntimer" },
    { id: "tags", label: "Tags" },
    { id: "exam", label: "Prüfungen" },
    { id: "archive", label: "Archiv" },
    { id: "brand", label: "Stapelweise" },
    { id: "settings", label: "Einstellungen" },
  ];
  let syncingObsidian = $state(false);
  let obsidianSyncMessage = $state<string | null>(null);
  let obsidianSyncFailed = $state(false);

  $effect(() => {
    s.load();
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function thresholdLabel(v: number): string {
    switch (v) {
      case 1: return "Mild";
      case 2: return "Moderat";
      case 3: return "Standard";
      case 4: return "Streng";
      default: return String(v);
    }
  }

  function toggleAnimation(key: AnimationSetting) {
    s.save({ [key]: !s.current[key] } as Partial<AppSettings>);
  }

  function updateTimerMinimum(value: number) {
    const minimum = Math.min(475, Math.max(1, Math.round(value)));
    s.save({
      timer_min_minutes: minimum,
      timer_max_minutes: Math.max(s.current.timer_max_minutes, minimum + 5),
    });
  }

  function updateTimerMaximum(value: number) {
    const maximum = Math.min(480, Math.max(s.current.timer_min_minutes + 5, Math.round(value)));
    s.save({ timer_max_minutes: maximum });
  }

  function colorForSlot(slot: ModuleColorSlot): string {
    if (s.current.color_theme === "custom") {
      return slot === "primary" ? s.current.custom_primary_color : s.current.custom_secondary_color;
    }
    const theme = colorThemes.find((candidate) => candidate.id === s.current.color_theme) ?? colorThemes[0];
    return theme[slot];
  }

  function setCustomPrimaryColor(color: string) {
    s.save({ color_theme: "custom", custom_primary_color: color });
  }

  function setCustomSecondaryColor(color: string) {
    s.save({ color_theme: "custom", custom_secondary_color: color });
  }

  async function syncObsidianVault() {
    if (!s.current.obsidian_vault_path || syncingObsidian) return;
    syncingObsidian = true;
    obsidianSyncMessage = null;
    try {
      await api.syncObsidianVault(s.current.obsidian_vault_path, "Obsidian Import");
      obsidianSyncFailed = false;
      obsidianSyncMessage = t("Obsidian-Vault importiert.");
    } catch {
      obsidianSyncFailed = true;
      obsidianSyncMessage = t("Obsidian-Synchronisierung fehlgeschlagen.");
    } finally {
      syncingObsidian = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="app-container flex items-center gap-3 pt-6 pb-4 sm:pt-8">
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
      {t("settings")}
    </h1>
  </div>

  <div class="app-container flex-1 overflow-y-auto pb-8 space-y-8">
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">{t("language")}</h2>
      <div class="max-w-sm">
        <label for="ui-language" class="text-sm font-medium text-primary dark:text-primary-dark">{t("language")}</label>
        <p class="text-xs text-secondary mb-2">{t("languageDescription")}</p>
        <select
          id="ui-language"
          value={s.current.ui_language}
          onchange={(event) => s.save({ ui_language: (event.target as HTMLSelectElement).value as UiLanguage })}
          class="w-full rounded-md border border-[#d8dee8] bg-transparent px-3 py-2 text-sm text-primary dark:border-[#303744] dark:text-primary-dark"
        >
          {#each uiLanguageOptions as language}
            <option value={language.code}>{language.label}</option>
          {/each}
        </select>
      </div>
    </section>

    <!-- Section: Erscheinungsbild -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">{t("appearance")}</h2>
      <div class="space-y-4">
        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">{t("Farbwelt")}</span>
          <p class="text-xs text-secondary mb-2">Wähle ein Farbduo oder stelle ein eigenes Duo zusammen.</p>
          <div class="grid max-w-2xl gap-2 sm:grid-cols-2">
            {#each colorThemes as colorTheme}
              <button
                onclick={() => s.save({ color_theme: colorTheme.id as ColorTheme })}
                aria-pressed={s.current.color_theme === colorTheme.id}
                class="flex min-h-14 items-center gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.color_theme === colorTheme.id
                  ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                  : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
              >
                <span class="flex shrink-0 -space-x-1" aria-hidden="true">
                  <span class="h-6 w-6 rounded-full border-2 border-white dark:border-[#171B24]" style="background-color: {colorTheme.primary}"></span>
                  <span class="h-6 w-6 rounded-full border-2 border-white dark:border-[#171B24]" style="background-color: {colorTheme.secondary}"></span>
                </span>
                <span class="text-sm font-semibold">{t(colorTheme.label)}</span>
              </button>
            {/each}
            <button
              onclick={() => s.save({ color_theme: "custom" })}
              aria-pressed={s.current.color_theme === "custom"}
              class="flex min-h-14 items-center gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.color_theme === 'custom'
                ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
            >
              <span class="flex shrink-0 -space-x-1" aria-hidden="true">
                <span class="h-6 w-6 rounded-full border-2 border-white dark:border-[#171B24]" style="background-color: {s.current.custom_primary_color}"></span>
                <span class="h-6 w-6 rounded-full border-2 border-white dark:border-[#171B24]" style="background-color: {s.current.custom_secondary_color}"></span>
              </span>
              <span class="text-sm font-semibold">Eigenes Farbduo</span>
            </button>
          </div>

          {#if s.current.color_theme === "custom"}
            <div class="mt-3 max-w-2xl rounded-lg border border-current/10 bg-white/35 p-3 dark:bg-white/5">
              <div class="grid gap-3 sm:grid-cols-2">
                <label class="flex items-center gap-3 text-sm font-semibold text-primary dark:text-primary-dark">
                  <input
                    type="color"
                    value={s.current.custom_primary_color}
                    onchange={(event) => setCustomPrimaryColor(event.currentTarget.value)}
                    class="h-9 w-11 cursor-pointer rounded border border-current/15 bg-transparent p-0.5"
                  />
                  <span>Erste Farbe <span class="font-mono text-xs font-normal text-secondary">{s.current.custom_primary_color}</span></span>
                </label>
                <label class="flex items-center gap-3 text-sm font-semibold text-primary dark:text-primary-dark">
                  <input
                    type="color"
                    value={s.current.custom_secondary_color}
                    onchange={(event) => setCustomSecondaryColor(event.currentTarget.value)}
                    class="h-9 w-11 cursor-pointer rounded border border-current/15 bg-transparent p-0.5"
                  />
                  <span>Zweite Farbe <span class="font-mono text-xs font-normal text-secondary">{s.current.custom_secondary_color}</span></span>
                </label>
              </div>
              <div class="mt-3 border-t border-current/10 pt-3">
                <p class="text-xs font-semibold text-secondary">Passende Kontrastfarben zur ersten Farbe</p>
                <div class="mt-2 flex flex-wrap gap-2">
                  {#each colorSuggestions(s.current.custom_primary_color) as suggestion}
                    <button
                      onclick={() => setCustomSecondaryColor(suggestion.color)}
                      class="inline-flex items-center gap-2 rounded-md border border-current/15 px-2.5 py-1.5 text-xs font-semibold text-primary transition-colors hover:border-accent-correct/45 dark:text-primary-dark"
                      aria-label={`${suggestion.label}: ${suggestion.color} als zweite Farbe wählen`}
                    >
                      <span class="h-4 w-4 rounded-full border border-black/10" style="background-color: {suggestion.color}"></span>
                      {suggestion.label}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}

          <div class="mt-4 max-w-2xl border-t border-current/10 pt-4">
            <span class="text-sm font-medium text-primary dark:text-primary-dark">Modulfarben zuordnen</span>
            <p class="mb-3 text-xs text-secondary">Lege für jedes Dashboard-Modul fest, welche der beiden Farben es verwendet.</p>
            <div class="grid gap-2 sm:grid-cols-2">
              {#each moduleColorTargets as target}
                <div class="flex items-center justify-between gap-2 rounded-md border border-current/10 px-3 py-2">
                  <span class="text-xs font-semibold text-primary dark:text-primary-dark">{target.label}</span>
                  <div class="flex overflow-hidden rounded-md border border-current/15" role="group" aria-label={`Farbe für ${target.label}`}>
                    {#each (["primary", "secondary"] as ModuleColorSlot[]) as slot}
                      <button
                        onclick={() => s.setModuleColor(target.id, slot)}
                        aria-pressed={s.moduleColorFor(target.id) === slot}
                        class="flex items-center gap-1 px-2 py-1 text-[10px] font-semibold transition-colors {s.moduleColorFor(target.id) === slot ? 'text-white' : 'text-secondary hover:bg-current/5'}"
                        style={s.moduleColorFor(target.id) === slot ? `background-color: ${colorForSlot(slot)}` : ""}
                      >
                        <span class="h-2 w-2 rounded-full border border-current/20" style={`background-color: ${colorForSlot(slot)}`}></span>
                        {slot === "primary" ? "Erste" : "Zweite"}
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>

        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">{t("Stapel-Vorschau")}</span>
          <p class="text-xs text-secondary mb-2">{t("Zeige die ersten drei Karteikarten dekorativ hinter ihren Stapelmodulen an.")}</p>
          <button
            onclick={() => s.save({ show_deck_card_previews: !s.current.show_deck_card_previews })}
            aria-pressed={s.current.show_deck_card_previews}
            class="rounded-button px-4 py-1.5 text-sm font-medium transition-transform hover:scale-[1.02] {s.current.show_deck_card_previews
              ? 'bg-accent-correct text-white'
              : 'bg-white/40 dark:bg-white/10 text-secondary hover:text-primary dark:hover:text-primary-dark'}"
          >
            {s.current.show_deck_card_previews ? t("Anzeigen") : t("Ausblenden")}
          </button>
        </div>

        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">{t("Überschriften-Schriftart")}</span>
          <p class="text-xs text-secondary mb-2">{t("Schriftart für Überschriften, Zähler und das Stapelweise-Logo.")}</p>
          <div class="grid max-w-2xl gap-2 sm:grid-cols-2">
            <button
              onclick={() => s.save({ pixel_font: "press-start" })}
              aria-pressed={s.current.pixel_font === "press-start"}
              class="flex min-h-16 items-center justify-between gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.pixel_font === 'press-start'
                ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
            >
              <span class="text-sm font-semibold">Press Start 2P</span>
              <span class="font-pixel text-base text-primary dark:text-primary-dark" style="font-family: 'Press Start 2P'">Aa 12</span>
            </button>
            <button
              onclick={() => s.save({ pixel_font: "silkscreen" })}
              aria-pressed={s.current.pixel_font === "silkscreen"}
              class="flex min-h-16 items-center justify-between gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.pixel_font === 'silkscreen'
                ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
            >
              <span class="text-sm font-semibold">Silkscreen</span>
              <span class="text-lg font-bold text-primary dark:text-primary-dark" style="font-family: 'Silkscreen'">Aa 12</span>
            </button>
            <button
              onclick={() => s.save({ pixel_font: "source-sans" })}
              aria-pressed={s.current.pixel_font === "source-sans"}
              class="flex min-h-16 items-center justify-between gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.pixel_font === 'source-sans'
                ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
            >
              <span class="text-sm font-semibold">Source Sans 3</span>
              <span class="text-lg font-bold text-primary dark:text-primary-dark" style="font-family: 'Source Sans 3'">Aa 12</span>
            </button>
            <button
              onclick={() => s.save({ pixel_font: "source-serif" })}
              aria-pressed={s.current.pixel_font === "source-serif"}
              class="flex min-h-16 items-center justify-between gap-3 rounded-md border px-3 py-2 text-left transition-colors {s.current.pixel_font === 'source-serif'
                ? 'border-accent-correct bg-accent-correct/10 text-primary dark:text-primary-dark'
                : 'border-[#d8dee8] bg-white/40 text-secondary hover:border-accent-correct/45 hover:text-primary dark:border-[#303744] dark:bg-white/5 dark:hover:text-primary-dark'}"
            >
              <span class="text-sm font-semibold">Source Serif 4</span>
              <span class="text-lg font-semibold text-primary dark:text-primary-dark" style="font-family: 'Source Serif 4'">Aa 12</span>
            </button>
          </div>
        </div>

        <!-- Font Family -->
        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">Karten-Schriftart</span>
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
              Sans (Source Sans 3)
            </button>
          </div>
        </div>

        <!-- Font Size -->
        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">Karten-Schriftgröße</span>
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

        <!-- Learning Animations -->
        <div class="max-w-2xl space-y-3 pt-1">
          <div class="flex items-center justify-between gap-6">
            <div>
              <span class="text-sm font-medium text-primary dark:text-primary-dark">Lernanimationen</span>
              <p class="text-xs text-secondary">Alle Bewegungen im Lernmodus.</p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={s.current.learning_animations}
              aria-label="Alle Lernanimationen"
              onclick={() => s.save({ learning_animations: !s.current.learning_animations })}
              class="relative h-6 w-11 shrink-0 rounded-full transition-colors {s.current.learning_animations ? 'bg-accent-correct' : 'bg-secondary/35'}"
            >
              <span
                class="absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white shadow-sm transition-transform {s.current.learning_animations ? 'translate-x-5' : 'translate-x-0'}"
              ></span>
            </button>
          </div>

          <div class="ml-2 space-y-2 border-l border-secondary/20 pl-4 {s.current.learning_animations ? '' : 'opacity-45'}">
            {#each animationOptions as option}
              <div class="flex min-h-8 items-center justify-between gap-6">
                <span class="text-xs font-medium text-primary dark:text-primary-dark">{option.label}</span>
                <button
                  type="button"
                  role="switch"
                  disabled={!s.current.learning_animations}
                  aria-checked={s.current[option.key]}
                  aria-label={option.label}
                  onclick={() => toggleAnimation(option.key)}
                  class="relative h-5 w-9 shrink-0 rounded-full transition-colors disabled:cursor-not-allowed {s.current[option.key] ? 'bg-accent-correct' : 'bg-secondary/35'}"
                >
                  <span
                    class="absolute left-0.5 top-0.5 h-4 w-4 rounded-full bg-white shadow-sm transition-transform {s.current[option.key] ? 'translate-x-4' : 'translate-x-0'}"
                  ></span>
                </button>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <!-- Section: Lernerfahrung -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">{t("learningExperience")}</h2>
      <div class="space-y-5">
        <!-- Session Limit -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label for="input_session_limit" class="text-sm font-medium text-primary dark:text-primary-dark">Karten pro Session</label>
            <span class="text-sm font-semibold text-accent-correct">{s.current.session_limit}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Maximale Anzahl fälliger Karten, die pro Lern-Durchgang gezeigt werden.</p>
          <input
            id="input_session_limit"
            type="range"
            min="5"
            max="200"
            step="5"
            value={s.current.session_limit}
            onchange={(e) => s.save({ session_limit: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>5</span>
            <span>200</span>
          </div>
        </div>

        <div class="border-t border-current/10 pt-5">
          <div class="flex items-center justify-between gap-4">
            <div>
              <p class="text-sm font-medium text-primary dark:text-primary-dark">Lerntimer</p>
              <p class="mt-1 text-xs text-secondary">Skalierung und Wertebereich des Zeitreglers.</p>
            </div>
            <div class="flex overflow-hidden rounded-md border border-current/15" role="group" aria-label="Timer-Skalierung">
              <button
                onclick={() => s.save({ timer_slider_scale: "linear" })}
                class="px-3 py-1.5 text-xs font-semibold transition-colors {s.current.timer_slider_scale === 'linear' ? 'bg-accent-correct text-white' : 'text-secondary hover:bg-current/5'}"
                aria-pressed={s.current.timer_slider_scale === "linear"}
              >Linear</button>
              <button
                onclick={() => s.save({ timer_slider_scale: "logarithmic" })}
                class="px-3 py-1.5 text-xs font-semibold transition-colors {s.current.timer_slider_scale === 'logarithmic' ? 'bg-accent-correct text-white' : 'text-secondary hover:bg-current/5'}"
                aria-pressed={s.current.timer_slider_scale === "logarithmic"}
              >Logarithmisch</button>
            </div>
          </div>
          <div class="mt-4 grid grid-cols-2 gap-3">
            <label class="block text-xs font-semibold text-secondary">
              Minimum (Min.)
              <input
                type="number"
                min="1"
                max={s.current.timer_max_minutes - 5}
                step="5"
                value={s.current.timer_min_minutes}
                onchange={(event) => updateTimerMinimum(Number(event.currentTarget.value))}
                class="module-accent-input mt-1.5 w-full rounded-md px-3 py-2 text-sm font-medium"
              />
            </label>
            <label class="block text-xs font-semibold text-secondary">
              Maximum (Min.)
              <input
                type="number"
                min={s.current.timer_min_minutes + 5}
                max="480"
                step="5"
                value={s.current.timer_max_minutes}
                onchange={(event) => updateTimerMaximum(Number(event.currentTarget.value))}
                class="module-accent-input mt-1.5 w-full rounded-md px-3 py-2 text-sm font-medium"
              />
            </label>
          </div>
        </div>

        <!-- Pass Threshold -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label for="input_sm2_pass_threshold" class="text-sm font-medium text-primary dark:text-primary-dark">SM-2 Strenge</label>
            <span class="text-sm font-semibold text-accent-correct">{thresholdLabel(s.current.sm2_pass_threshold)}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Welche Mindest-Bewertung gilt als "richtig" (bestanden)? Höher = strenger.</p>
          <input
            id="input_sm2_pass_threshold"
            type="range"
            min="1"
            max="4"
            step="1"
            value={s.current.sm2_pass_threshold}
            onchange={(e) => s.save({ sm2_pass_threshold: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>1 ({t("mild")})</span>
            <span>3 ({t("standard")})</span>
            <span>4 ({t("streng")})</span>
          </div>
        </div>

        <!-- Initial Ease Factor -->
        <div>
          <div class="flex items-center justify-between mb-1">
            <label for="input_sm2_initial_ef" class="text-sm font-medium text-primary dark:text-primary-dark">Start-Ease-Faktor</label>
            <span class="text-sm font-semibold text-accent-correct">{s.current.sm2_initial_ef.toFixed(1)}</span>
          </div>
          <p class="text-xs text-secondary mb-2">Startwert für die SM-2-Intervall-Berechnung. Höher = längere Intervalle.</p>
          <input
            id="input_sm2_initial_ef"
            type="range"
            min="1.3"
            max="3.0"
            step="0.1"
            value={s.current.sm2_initial_ef}
            onchange={(e) => s.save({ sm2_initial_ef: Number((e.target as HTMLInputElement).value) })}
            class="w-full accent-accent-correct"
          />
          <div class="flex justify-between text-xs text-secondary mt-0.5">
            <span>1.3</span>
            <span>2.5 ({t("standard")})</span>
            <span>3.0</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Section: Integrationen -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">Integrationen</h2>
      <div class="space-y-4">
        <IntegrationImports />

        <!-- Obsidian Sync -->
        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">Obsidian Vault Sync</span>
          <p class="text-xs text-secondary mb-2">Importiere Markdown-Karten aus einem lokalen Ordner/Vault.</p>
          <div class="space-y-3 mb-3">
            <div>
              <label for="input_obsidian_vault_path" class="text-xs font-medium text-secondary block mb-1">Vault Pfad (Absolut)</label>
              <input
                id="input_obsidian_vault_path"
                type="text"
                placeholder="C:/Users/name/Documents/Obsidian"
                value={s.current.obsidian_vault_path}
                oninput={(e) => s.save({ obsidian_vault_path: (e.target as HTMLInputElement).value })}
                class="w-full glass rounded-card px-3 py-1.5 text-sm border border-secondary/30 bg-transparent text-primary dark:text-primary-dark outline-none focus:border-accent-correct"
              />
            </div>
            <div>
              <label for="input_obsidian_flashcard_tag" class="text-xs font-medium text-secondary block mb-1">Flashcard Tag</label>
              <input
                id="input_obsidian_flashcard_tag"
                type="text"
                placeholder="#flashcard"
                value={s.current.obsidian_flashcard_tag}
                oninput={(e) => s.save({ obsidian_flashcard_tag: (e.target as HTMLInputElement).value })}
                class="w-full glass rounded-card px-3 py-1.5 text-sm border border-secondary/30 bg-transparent text-primary dark:text-primary-dark outline-none focus:border-accent-correct"
              />
            </div>
          </div>
          <div class="flex items-center justify-between">
            <p class="text-[10px] max-w-[70%] {obsidianSyncFailed ? 'text-accent-incorrect' : 'text-secondary'}" aria-live="polite">{obsidianSyncMessage ?? t("Sucht nach Markdown-Dateien mit dem konfigurierten Tag. Dateiname = Vorderseite, Inhalt = Rückseite.")}</p>
            <button
              onclick={() => void syncObsidianVault()}
              disabled={!s.current.obsidian_vault_path || syncingObsidian}
              class="rounded-button bg-accent-correct text-white px-4 py-1.5 text-sm font-medium hover:scale-[1.02] transition-transform disabled:cursor-not-allowed disabled:opacity-45"
            >
              {syncingObsidian ? t("Synchronisiert...") : t("Sync starten")}
            </button>
          </div>
        </div>

        <!-- MCP Server Status -->
        <div class="pt-4 border-t border-secondary/20">
          <span class="text-sm font-medium text-primary dark:text-primary-dark">URL-Schema</span>
          <div class="mt-2 grid gap-1 text-[11px] text-secondary font-mono">
            <code>stapelweise://deck/new?name=Biologie</code>
            <code>stapelweise://deck/open?deck=Biologie</code>
            <code>stapelweise://card/new?deck=Biologie&amp;front=...&amp;back=...</code>
          </div>
        </div>

        <div class="pt-4 border-t border-secondary/20">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium text-primary dark:text-primary-dark">MCP Server Integration</span>
            <span class="text-xs font-bold px-2 py-0.5 rounded bg-accent-correct/20 text-accent-correct">Bereit</span>
          </div>
          <p class="text-xs text-secondary mb-3">Erlaube KI-Assistenten (z.B. Claude Desktop) direkten Zugriff auf deine Karten. Die KI kann dich abfragen, Zusammenfassungen erstellen oder Karten für dich anlegen.</p>
          <div class="glass rounded-card p-3 bg-black/5 dark:bg-black/20">
            <p class="text-xs font-semibold mb-1 text-primary dark:text-primary-dark">Claude Desktop Konfiguration:</p>
            <p class="text-[10px] text-secondary mb-2">Füge Folgendes in deine <code>claude_desktop_config.json</code> ein:</p>
            <pre class="text-[10px] font-mono text-secondary overflow-x-auto whitespace-pre-wrap">
{`"mcpServers": {
  "stapelweise": {
    "command": "pfad/zur/stapelweise.exe",
    "args": ["--mcp"]
  }
}`}
            </pre>
            <p class="text-[10px] text-secondary mt-2">Das `--mcp` Flag startet Stapelweise im Hintergrund ohne UI, nur als Server.</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Section: Datenverwaltung -->
    <section>
      <h2 class="text-sm font-semibold text-secondary uppercase tracking-wider mb-4">Datenverwaltung</h2>
      <div class="space-y-4">
        <!-- Beispieldaten -->
        <div>
          <span class="text-sm font-medium text-primary dark:text-primary-dark">Beispieldaten</span>
          <p class="text-xs text-secondary mb-3">{t("Lade 7 thematische Muster-Stapel (Grammatik, Geschichte, Biologie, LaTeX, Stapelweise-Tipps, Compute Engine sowie Logik & Mengenlehre) mit verschiedenen Lernzuständen und Kartentypen in deine Bibliothek.")}</p>
          <button
            onclick={async () => {
              try {
                await deckStore.seed();
                alert("Beispieldaten erfolgreich geladen! Kehre zur Hauptübersicht zurück, um sie zu sehen.");
              } catch (e: any) {
                alert("Fehler beim Laden: " + e);
              }
            }}
            class="rounded-button bg-white/40 dark:bg-white/10 text-primary dark:text-primary-dark px-4 py-1.5 text-sm font-medium hover:bg-white/70 dark:hover:bg-white/20 transition-all border border-white/10"
          >
            Beispieldaten laden
          </button>
        </div>
      </div>
    </section>
  </div>
</div>
