<script lang="ts">
  import { t } from "$lib/i18n";
  import type { SymbolicEvaluationResult, SymbolicEvaluationStatus } from "$lib/math-evaluation";

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

  const resultStyles: Record<SymbolicEvaluationStatus, string> = {
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

  function resultStyle(status: SymbolicEvaluationStatus) {
    return resultStyles[status];
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape") {
      (event.currentTarget as HTMLTextAreaElement).blur();
    }
  }
</script>

<div class="w-full max-w-2xl">
  <label class="mb-1 block text-xs font-medium text-secondary" for="free-text-response">
    {t("freeTextAnswer")}
  </label>
  <textarea
    id="free-text-response"
    value={value}
    disabled={disabled}
    rows="2"
    placeholder={evaluationMode === "symbolic" ? t("mathLatexPlaceholder") : t("freeTextPlaceholder")}
    oninput={(event) => onChange((event.target as HTMLTextAreaElement).value)}
    onkeydown={handleKeydown}
    class="w-full resize-none rounded-md border border-secondary/25 bg-white/55 px-3 py-2 text-sm text-primary outline-none focus:border-accent-correct disabled:cursor-default disabled:bg-secondary/10 dark:bg-black/20 dark:text-primary-dark"
  ></textarea>
  {#if result}
    <p class="mt-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium {resultStyle(result.status as SymbolicEvaluationStatus)}">
      {resultLabel(result.status)}
    </p>
  {/if}
</div>
