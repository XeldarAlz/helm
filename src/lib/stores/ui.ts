import { writable } from "svelte/store";

export type ViewName =
  | "dashboard"
  | "chat"
  | "orchestration"
  | "documents"
  | "code"
  | "git"
  | "history"
  | "settings";

export const activeView = writable<ViewName>("dashboard");
export const sidebarCollapsed = writable(false);
export const activeModal = writable<string | null>(null);
