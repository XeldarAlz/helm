import type { TransitionConfig } from "svelte/transition";

export function fadeIn(
  _node: Element,
  { duration = 200, delay = 0 }: { duration?: number; delay?: number } = {},
): TransitionConfig {
  return {
    duration,
    delay,
    css: (t) => `opacity: ${t}`,
  };
}

export function fadeOut(
  _node: Element,
  { duration = 150, delay = 0 }: { duration?: number; delay?: number } = {},
): TransitionConfig {
  return {
    duration,
    delay,
    css: (t) => `opacity: ${t}`,
  };
}

export function fadeScale(
  _node: Element,
  {
    duration = 200,
    delay = 0,
    startScale = 0.95,
  }: { duration?: number; delay?: number; startScale?: number } = {},
): TransitionConfig {
  return {
    duration,
    delay,
    css: (t) => {
      const scale = startScale + (1 - startScale) * t;
      return `opacity: ${t}; transform: scale(${scale})`;
    },
  };
}
