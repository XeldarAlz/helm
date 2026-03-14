import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { initKeyboardShortcuts, destroyKeyboardShortcuts } from "../keyboard";

// Mock the stores that keyboard.ts imports
vi.mock("$lib/stores/ui", () => {
  const set = vi.fn();
  const update = vi.fn();
  return {
    activeView: { set, update, subscribe: vi.fn() },
    sidebarCollapsed: { set, update: vi.fn((fn: (v: boolean) => boolean) => fn(false)), subscribe: vi.fn() },
  };
});

describe("keyboard shortcuts", () => {
  beforeEach(() => {
    destroyKeyboardShortcuts();
  });

  afterEach(() => {
    destroyKeyboardShortcuts();
  });

  it("adds keydown listener on init", () => {
    const spy = vi.spyOn(window, "addEventListener");
    initKeyboardShortcuts();
    expect(spy).toHaveBeenCalledWith("keydown", expect.any(Function));
    spy.mockRestore();
  });

  it("removes keydown listener on destroy", () => {
    initKeyboardShortcuts();
    const spy = vi.spyOn(window, "removeEventListener");
    destroyKeyboardShortcuts();
    expect(spy).toHaveBeenCalledWith("keydown", expect.any(Function));
    spy.mockRestore();
  });

  it("does not double-register on repeated init calls", () => {
    const spy = vi.spyOn(window, "addEventListener");
    initKeyboardShortcuts();
    initKeyboardShortcuts();
    const keydownCalls = spy.mock.calls.filter((c) => c[0] === "keydown");
    expect(keydownCalls).toHaveLength(1);
    spy.mockRestore();
  });

  it("suppresses shortcuts when target is INPUT", () => {
    initKeyboardShortcuts();
    const input = document.createElement("input");
    document.body.appendChild(input);

    const event = new KeyboardEvent("keydown", {
      key: "n",
      metaKey: true,
      bubbles: true,
    });
    const preventSpy = vi.spyOn(event, "preventDefault");

    input.dispatchEvent(event);
    expect(preventSpy).not.toHaveBeenCalled();

    document.body.removeChild(input);
  });

  it("suppresses shortcuts when target is TEXTAREA", () => {
    initKeyboardShortcuts();
    const textarea = document.createElement("textarea");
    document.body.appendChild(textarea);

    const event = new KeyboardEvent("keydown", {
      key: ",",
      metaKey: true,
      bubbles: true,
    });
    const preventSpy = vi.spyOn(event, "preventDefault");

    textarea.dispatchEvent(event);
    expect(preventSpy).not.toHaveBeenCalled();

    document.body.removeChild(textarea);
  });
});
