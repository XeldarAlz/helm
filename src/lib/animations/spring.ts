export const springs = {
  snappy: { stiffness: 0.15, damping: 0.8 },
  smooth: { stiffness: 0.1, damping: 0.85 },
  bouncy: { stiffness: 0.2, damping: 0.6 },
  gentle: { stiffness: 0.08, damping: 0.9 },
  stiff: { stiffness: 0.3, damping: 0.95 },
} as const;

export type SpringPreset = keyof typeof springs;
