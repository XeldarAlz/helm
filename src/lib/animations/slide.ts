import type { TransitionConfig } from "svelte/transition";

type Direction = "left" | "right" | "up" | "down";

const offsets: Record<Direction, { x: number; y: number }> = {
  left: { x: -20, y: 0 },
  right: { x: 20, y: 0 },
  up: { x: 0, y: -20 },
  down: { x: 0, y: 20 },
};

export function slide(
  _node: Element,
  {
    direction = "up",
    duration = 200,
    delay = 0,
    distance = 20,
  }: {
    direction?: Direction;
    duration?: number;
    delay?: number;
    distance?: number;
  } = {},
): TransitionConfig {
  const dir = offsets[direction];
  const dx = dir.x < 0 ? -distance : dir.x > 0 ? distance : 0;
  const dy = dir.y < 0 ? -distance : dir.y > 0 ? distance : 0;

  return {
    duration,
    delay,
    css: (t) => {
      const x = dx * (1 - t);
      const y = dy * (1 - t);
      return `opacity: ${t}; transform: translate(${x}px, ${y}px)`;
    },
  };
}
