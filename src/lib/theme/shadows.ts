export const shadows = {
  sm: "0 1px 2px rgba(0, 0, 0, 0.3)",
  md: "0 4px 12px rgba(0, 0, 0, 0.4)",
  lg: "0 8px 24px rgba(0, 0, 0, 0.5)",
  xl: "0 16px 48px rgba(0, 0, 0, 0.6)",
} as const;

export const glows = {
  accent: "0 0 12px var(--color-accent-glow), 0 0 4px var(--color-accent-glow)",
  success: "0 0 12px rgba(63, 185, 80, 0.25)",
  error: "0 0 12px rgba(248, 81, 73, 0.25)",
} as const;
