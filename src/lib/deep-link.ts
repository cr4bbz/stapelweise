import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";

export type StapelweiseDeepLink =
  | { kind: "new-deck"; name: string }
  | { kind: "open-deck"; deck: string }
  | { kind: "new-card"; deck: string; front: string; back: string };

function value(params: URLSearchParams, key: string) {
  return (params.get(key) ?? "").trim();
}

export function parseDeepLink(rawUrl: string): StapelweiseDeepLink | null {
  let url: URL;
  try {
    url = new URL(rawUrl);
  } catch {
    return null;
  }
  if (url.protocol !== "stapelweise:") return null;

  const action = `${url.hostname}${url.pathname}`.replace(/^\/+|\/+$/g, "");
  if (action === "deck/new") return { kind: "new-deck", name: value(url.searchParams, "name") };
  if (action === "deck/open") {
    const deck = value(url.searchParams, "deck");
    return deck ? { kind: "open-deck", deck } : null;
  }
  if (action === "card/new") {
    const deck = value(url.searchParams, "deck");
    const front = value(url.searchParams, "front");
    const back = value(url.searchParams, "back");
    return deck && front && back ? { kind: "new-card", deck, front, back } : null;
  }
  return null;
}

export async function listenForDeepLinks(onLink: (link: StapelweiseDeepLink) => void) {
  const deliver = (urls: string[]) => urls.map(parseDeepLink).filter((link): link is StapelweiseDeepLink => link !== null).forEach(onLink);
  try {
    const initial = await getCurrent();
    if (initial) deliver(initial);
    return await onOpenUrl(deliver);
  } catch {
    return () => {};
  }
}
