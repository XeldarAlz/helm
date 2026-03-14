import type { TransitionConfig } from "svelte/transition";

export function staggeredItem(
  _node: Element,
  {
    index = 0,
    staggerDelay = 50,
    duration = 200,
    distance = 8,
  }: {
    index?: number;
    staggerDelay?: number;
    duration?: number;
    distance?: number;
  } = {},
): TransitionConfig {
  return {
    duration,
    delay: index * staggerDelay,
    css: (t) => {
      const y = distance * (1 - t);
      return `opacity: ${t}; transform: translateY(${y}px)`;
    },
  };
}
