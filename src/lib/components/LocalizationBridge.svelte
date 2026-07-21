<script lang="ts">
  import { onMount } from "svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { translateUiPhrase } from "$lib/i18n";

  const textSources = new WeakMap<Text, { source: string; applied: string }>();
  const attributeSources = new WeakMap<Element, Map<string, { source: string; applied: string }>>();
  const translatableAttributes = ["title", "aria-label", "placeholder"];
  let observer: MutationObserver | null = null;

  function translateValue(value: string) {
    const match = value.match(/^(\s*)([\s\S]*?)(\s*)$/);
    if (!match) return value;
    const translated = translateUiPhrase(match[2]);
    if (translated !== match[2]) return `${match[1]}${translated}${match[3]}`;

    // Keep familiar leading symbols while translating their adjacent UI label.
    const symbolLabel = match[2].match(/^([^\p{L}\p{N}]*)([\s\S]+)$/u);
    if (!symbolLabel) return value;
    const translatedLabel = translateUiPhrase(symbolLabel[2]);
    return translatedLabel === symbolLabel[2]
      ? value
      : `${match[1]}${symbolLabel[1]}${translatedLabel}${match[3]}`;
  }

  function shouldSkip(element: Element | null) {
    return Boolean(element?.closest("script, style, code, pre, textarea, [data-user-content]"));
  }

  function translateTextNode(node: Text) {
    if (shouldSkip(node.parentElement)) return;
    const existing = textSources.get(node);
    const source = !existing || node.data !== existing.applied ? node.data : existing.source;
    const applied = translateValue(source);
    textSources.set(node, { source, applied });
    if (node.data !== applied) node.data = applied;
  }

  function translateAttributes(element: Element) {
    if (shouldSkip(element)) return;
    const entries = attributeSources.get(element) ?? new Map<string, { source: string; applied: string }>();
    for (const name of translatableAttributes) {
      const value = element.getAttribute(name);
      if (value === null) continue;
      const existing = entries.get(name);
      const source = !existing || value !== existing.applied ? value : existing.source;
      const applied = translateValue(source);
      entries.set(name, { source, applied });
      if (value !== applied) element.setAttribute(name, applied);
    }
    attributeSources.set(element, entries);
  }

  function translateTree(root: Node) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT);
    let current: Node | null = root;
    while (current) {
      if (current.nodeType === Node.TEXT_NODE) translateTextNode(current as Text);
      if (current.nodeType === Node.ELEMENT_NODE) translateAttributes(current as Element);
      current = walker.nextNode();
    }
  }

  $effect(() => {
    settingsStore.current.ui_language;
    if (typeof document !== "undefined") translateTree(document.body);
  });

  onMount(() => {
    translateTree(document.body);
    observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === "characterData") translateTextNode(mutation.target as Text);
        if (mutation.type === "attributes") translateAttributes(mutation.target as Element);
        mutation.addedNodes.forEach(translateTree);
      }
    });
    observer.observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: translatableAttributes });
    return () => observer?.disconnect();
  });
</script>
