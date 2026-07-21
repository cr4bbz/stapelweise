<script lang="ts">
  import { tick } from "svelte";
  import { t } from "$lib/i18n";
  import { renderLatexExpression } from "$lib/math";
  import type { SymbolicEvaluationResult } from "$lib/math-evaluation";

  let {
    value = "",
    disabled = false,
    evaluationMode = "manual",
    result = null,
    onChange = (_value: string) => {},
  } = $props<{
    value?: string;
    disabled?: boolean;
    evaluationMode?: "manual" | "symbolic";
    result?: SymbolicEvaluationResult | null;
    onChange?: (value: string) => void;
  }>();

  let textarea = $state<HTMLTextAreaElement>();
  let previewHtml = $state("");

  const mathTools = [
    { type: "fraction", label: "mathToolFraction", symbol: "a/b" },
    { type: "exponent", label: "mathToolExponent", symbol: "x^n" },
    { type: "root", label: "mathToolRoot", symbol: "sqrt" },
    { type: "pi", label: "mathToolPi", symbol: "pi" },
    { type: "integral", label: "mathToolIntegral", symbol: "int" },
  ];

  const resultStyles: Record<SymbolicEvaluationResult["status"], string> = {
    checking: "border-secondary/30 bg-secondary/10 text-secondary",
    empty: "border-secondary/30 bg-secondary/10 text-secondary",
    equivalent: "border-accent-easy/40 bg-accent-easy/10 text-accent-easy",
    "not-equivalent": "border-accent-incorrect/40 bg-accent-incorrect/10 text-accent-incorrect",
    "invalid-input": "border-accent-hard/40 bg-accent-hard/10 text-accent-hard",
    undecidable: "border-accent-hard/40 bg-accent-hard/10 text-accent-hard",
    unavailable: "border-secondary/30 bg-secondary/10 text-secondary",
  };

  function resultLabel(status: SymbolicEvaluationResult["status"]) {
    const labels: Record<SymbolicEvaluationResult["status"], string> = {
      checking: "mathChecking",
      empty: "mathEmptyAnswer",
      equivalent: "mathEquivalent",
      "not-equivalent": "mathNotEquivalent",
      "invalid-input": "mathInvalidInput",
      undecidable: "mathUndecidable",
      unavailable: "mathUnavailable",
    };
    return t(labels[status]);
  }

  function resultStyle(status: SymbolicEvaluationResult["status"]) {
    return resultStyles[status];
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      (event.currentTarget as HTMLTextAreaElement).blur();
    }
  }

  function mathToolInsertion(type: string, selection: string) {
    if (type === "fraction") {
      return selection
        ? { latex: `\\frac{${selection}}{}`, cursorOffset: selection.length + 8 }
        : { latex: "\\frac{}{}", cursorOffset: 6 };
    }
    if (type === "exponent") {
      return selection
        ? { latex: `${selection}^{}`, cursorOffset: selection.length + 2 }
        : { latex: "^{}", cursorOffset: 2 };
    }
    if (type === "root") {
      return selection
        ? { latex: `\\sqrt{${selection}}`, cursorOffset: selection.length + 7 }
        : { latex: "\\sqrt{}", cursorOffset: 6 };
    }
    if (type === "integral") {
      return selection
        ? { latex: `\\int_{${selection}}^{}`, cursorOffset: selection.length + 9 }
        : { latex: "\\int_{}^{}", cursorOffset: 6 };
    }
    return { latex: "\\pi", cursorOffset: 3 };
  }

  async function insertMathTool(type: string) {
    const start = textarea?.selectionStart ?? value.length;
    const end = textarea?.selectionEnd ?? value.length;
    const insertion = mathToolInsertion(type, value.slice(start, end));
    onChange(`${value.slice(0, start)}${insertion.latex}${value.slice(end)}`);

    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(start + insertion.cursorOffset, start + insertion.cursorOffset);
  }

  $effect(() => {
    const source = value;
    const frame = requestAnimationFrame(() => {
      previewHtml = source.trim() ? renderLatexExpression(source) : "";
    });
    return () => cancelAnimationFrame(frame);
  });
</script>

<div class="w-full max-w-2xl">
  <label class="mb-1 block text-xs font-medium text-secondary" for="free-text-response">
    {t("freeTextAnswer")}
  </label>
  <textarea
    id="free-text-response"
    bind:this={textarea}
    value={value}
    disabled={disabled}
    rows="2"
    placeholder={evaluationMode === "symbolic" ? t("mathLatexPlaceholder") : t("freeTextPlaceholder")}
    oninput={(event) => onChange((event.target as HTMLTextAreaElement).value)}
    onkeydown={handleKeydown}
    class="w-full resize-none rounded-md border border-secondary/25 bg-white/55 px-3 py-2 text-sm text-primary outline-none focus:border-accent-correct disabled:cursor-default disabled:bg-secondary/10 dark:bg-black/20 dark:text-primary-dark"
  ></textarea>
  {#if evaluationMode === "symbolic" && !disabled}
    <div class="mt-1.5 flex flex-wrap gap-1" aria-label={t("mathTools")}>
      {#each mathTools as tool}
        <button
          type="button"
          onclick={() => insertMathTool(tool.type)}
          class="h-7 min-w-8 border border-secondary/25 bg-white/45 px-1.5 font-serif text-xs text-primary transition-colors hover:border-accent-correct hover:text-accent-correct dark:bg-black/15 dark:text-primary-dark"
          title={t(tool.label)}
          aria-label={t(tool.label)}
        >
          {tool.symbol}
        </button>
      {/each}
    </div>
  {/if}
  {#if evaluationMode === "symbolic" && previewHtml}
    <div role="math" aria-label={value} class="mt-2 border border-dashed border-accent-correct/40 bg-white/45 px-3 py-2 text-center text-sm text-primary dark:bg-black/15 dark:text-primary-dark">
      <span class="mb-1 block text-left text-[10px] font-medium uppercase text-secondary">{t("mathPreview")}</span>
      {@html previewHtml}
    </div>
  {/if}
  {#if result}
    <p class="mt-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium {resultStyle(result.status)}">
      {resultLabel(result.status)}
    </p>
  {/if}
</div>
