export const durations = {
  instant: 0,
  fast: 150,
  normal: 200,
  slow: 300,
  pageTransition: 400,
  splash: 800,
} as const;

export const easings = {
  easeOut: "cubic-bezier(0.0, 0.0, 0.2, 1)",
  easeIn: "cubic-bezier(0.4, 0.0, 1, 1)",
  easeInOut: "cubic-bezier(0.4, 0.0, 0.2, 1)",
  spring: "cubic-bezier(0.34, 1.56, 0.64, 1)",
} as const;
