const stylesheets = page => [...page.head.querySelectorAll('link[rel="stylesheet"]')];
const styleHref = link => link.getAttribute("href");
const inlineStyles = page => [...page.head.querySelectorAll(":scope > style")];
const loadingStyles = new Map();

async function loadStyles(page) {
  const current = new Set(stylesheets(document).map(styleHref));
  await Promise.all(stylesheets(page).map(source => {
    const href = styleHref(source);
    if (loadingStyles.has(href)) return loadingStyles.get(href);
    if (current.has(href)) return;
    const ready = new Promise((resolve, reject) => {
      const link = document.importNode(source, true);
      link.media = "not all";
      link.addEventListener("load", resolve, { once: true });
      link.addEventListener("error", () => {
        link.remove();
        reject(new Error("Reader stylesheet unavailable"));
      }, { once: true });
      document.head.insertBefore(link, document.head.querySelector('link[href="/assets/site.css"]'));
    }).finally(() => loadingStyles.delete(href));
    loadingStyles.set(href, ready);
    return ready;
  }));
}

export async function readWork(path, done) {
  let page = null;
  for (let attempt = 0; attempt < 2; attempt++) {
    try {
      const response = await fetch(path, {
        credentials: "same-origin",
        cache: attempt ? "reload" : "default",
      });
      if (response.status >= 500) throw new Error("Reader page temporarily unavailable");
      if (!response.ok || new URL(response.url).origin !== location.origin) break;
      const next = new DOMParser().parseFromString(await response.text(), "text/html");
      if (!next.querySelector(".site-body-stack")) break;
      await loadStyles(next);
      page = next;
      break;
    } catch {
      if (!attempt) await new Promise(resolve => setTimeout(resolve, 180));
    }
  }
  // Let Rust fall back to normal link navigation if the retry also fails.
  done(page);
}

export function applyReaderStyles(page) {
  const active = new Set(stylesheets(page).map(styleHref));
  for (const link of stylesheets(document)) {
    link.media = active.has(styleHref(link)) ? "" : "not all";
  }
  for (const style of inlineStyles(document)) style.remove();
  for (const style of inlineStyles(page)) {
    document.head.insertBefore(document.importNode(style, true), document.head.querySelector('link[href="/assets/site.css"]'));
  }
}

let activeTurn;

export function stopWorkTurn() {
  activeTurn?.skipTransition();
  activeTurn = undefined;
  delete document.documentElement.dataset.readingTurn;
}

export function showWork(update, direction) {
  stopWorkTurn();
  if (!document.startViewTransition || matchMedia("(prefers-reduced-motion: reduce)").matches) {
    update();
    return;
  }
  document.documentElement.dataset.readingTurn = direction < 0 ? "prev" : "next";
  const turn = document.startViewTransition(update);
  activeTurn = turn;
  turn.ready.catch(() => {});
  const clean = () => {
    if (activeTurn === turn) {
      activeTurn = undefined;
      delete document.documentElement.dataset.readingTurn;
    }
  };
  turn.finished.then(clean, clean);
}
