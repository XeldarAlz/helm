import { activeView, sidebarCollapsed } from "$lib/stores/ui";
import { get } from "svelte/store";

interface Shortcut {
  key: string;
  meta: boolean;
  shift?: boolean;
  action: () => void;
}

const shortcuts: Shortcut[] = [
  {
    // Cmd+N — New session (go to dashboard)
    key: "n",
    meta: true,
    action: () => activeView.set("dashboard"),
  },
  {
    // Cmd+W — Close/end session (go to dashboard)
    key: "w",
    meta: true,
    action: () => activeView.set("dashboard"),
  },
  {
    // Cmd+, — Open settings
    key: ",",
    meta: true,
    action: () => activeView.set("settings"),
  },
  {
    // Cmd+1 — Dashboard
    key: "1",
    meta: true,
    action: () => activeView.set("dashboard"),
  },
  {
    // Cmd+2 — Chat
    key: "2",
    meta: true,
    action: () => activeView.set("chat"),
  },
  {
    // Cmd+3 — Orchestration
    key: "3",
    meta: true,
    action: () => activeView.set("orchestration"),
  },
  {
    // Cmd+4 — Documents
    key: "4",
    meta: true,
    action: () => activeView.set("documents"),
  },
  {
    // Cmd+5 — Code
    key: "5",
    meta: true,
    action: () => activeView.set("code"),
  },
  {
    // Cmd+6 — Git
    key: "6",
    meta: true,
    action: () => activeView.set("git"),
  },
  {
    // Cmd+7 — History
    key: "7",
    meta: true,
    action: () => activeView.set("history"),
  },
  {
    // Cmd+\ — Toggle sidebar
    key: "\\",
    meta: true,
    action: () => sidebarCollapsed.update((v) => !v),
  },
];

function handleKeydown(e: KeyboardEvent) {
  // Don't intercept if user is typing in an input/textarea
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

  for (const shortcut of shortcuts) {
    const metaMatch = shortcut.meta ? (e.metaKey || e.ctrlKey) : true;
    const shiftMatch = shortcut.shift ? e.shiftKey : !e.shiftKey;

    if (e.key === shortcut.key && metaMatch && shiftMatch) {
      e.preventDefault();
      shortcut.action();
      return;
    }
  }
}

let initialized = false;

export function initKeyboardShortcuts(): void {
  if (initialized) return;
  initialized = true;
  window.addEventListener("keydown", handleKeydown);
}

export function destroyKeyboardShortcuts(): void {
  window.removeEventListener("keydown", handleKeydown);
  initialized = false;
}
