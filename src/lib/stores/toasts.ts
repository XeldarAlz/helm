import { writable } from "svelte/store";

export interface ToastData {
  id: string;
  variant: "info" | "success" | "warning" | "error";
  message: string;
  duration: number;
  action?: { label: string; onclick: () => void };
}

export const toasts = writable<ToastData[]>([]);

let counter = 0;

export function addToast(
  message: string,
  variant: ToastData["variant"] = "info",
  duration = 4000,
  action?: ToastData["action"],
) {
  const id = `toast-${++counter}`;
  toasts.update((t) => [...t, { id, variant, message, duration, action }]);

  if (duration > 0) {
    setTimeout(() => dismissToast(id), duration);
  }

  return id;
}

export function dismissToast(id: string) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
