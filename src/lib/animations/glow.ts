export function glowPulse(node: Element, { color = "var(--color-accent-glow)", duration = 2000 } = {}) {
  const keyframes = [
    { boxShadow: `0 0 12px ${color}, 0 0 4px ${color}` },
    { boxShadow: `0 0 20px ${color}, 0 0 8px ${color}` },
    { boxShadow: `0 0 12px ${color}, 0 0 4px ${color}` },
  ];

  const animation = node.animate(keyframes, {
    duration,
    iterations: Infinity,
    easing: "ease-in-out",
  });

  return {
    destroy() {
      animation.cancel();
    },
  };
}
