import type { TransitionConfig } from "svelte/transition";

export function scaleIn(
  _node: Element,
  {
    duration = 200,
    delay = 0,
    start = 0.8,
  }: { duration?: number; delay?: number; start?: number } = {},
): TransitionConfig {
  return {
    duration,
    delay,
    css: (t) => {
      const scale = start + (1 - start) * t;
      return `opacity: ${t}; transform: scale(${scale})`;
    },
  };
}

export function scaleOut(
  _node: Element,
  {
    duration = 150,
    delay = 0,
    end = 1.02,
  }: { duration?: number; delay?: number; end?: number } = {},
): TransitionConfig {
  return {
    duration,
    delay,
    css: (t) => {
      const scale = 1 + (end - 1) * (1 - t);
      return `opacity: ${t}; transform: scale(${scale})`;
    },
  };
}
