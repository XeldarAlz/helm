export const fonts = {
  ui: '"Inter", system-ui, -apple-system, sans-serif',
  mono: '"JetBrains Mono", ui-monospace, monospace',
} as const;

export const fontSizes = {
  caption: "11px",
  body: "13px",
  subtitle: "15px",
  title: "18px",
  heading: "24px",
  display: "32px",
  logStream: "12px",
  codeBlock: "13px",
  chatCode: "14px",
} as const;

export const fontWeights = {
  regular: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
} as const;

export const lineHeights = {
  tight: 1.2,
  normal: 1.5,
  relaxed: 1.7,
} as const;
