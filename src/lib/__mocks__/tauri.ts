// Stub for @tauri-apps/api used in tests

export async function invoke(_cmd: string, _args?: Record<string, unknown>): Promise<unknown> {
  return undefined;
}

export async function listen(
  _event: string,
  _handler: (event: unknown) => void,
): Promise<() => void> {
  return () => {};
}

export type UnlistenFn = () => void;
