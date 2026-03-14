export function isMac(): boolean {
  return navigator.platform.toLowerCase().includes("mac");
}

export function modKey(): string {
  return isMac() ? "Cmd" : "Ctrl";
}
