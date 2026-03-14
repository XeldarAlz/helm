import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Session {
  id: string;
  phase: string;
  status: "active" | "ended" | "error";
  startedAt: string;
  endedAt?: string;
  messageCount: number;
  contextUsage: number;
}

export interface Message {
  id: string;
  role: "user" | "agent" | "system";
  content: string;
  timestamp: string;
}

export interface SessionSummary {
  id: string;
  phase: string;
  status: "active" | "ended" | "error";
  started_at: string;
  ended_at: string | null;
  message_count: number;
}

export const currentSession = writable<Session | null>(null);
export const messages = writable<Message[]>([]);
export const isAgentTyping = writable(false);
export const suggestedAnswers = writable<string[]>([]);

// ── Streaming state ─────────────────────────────────────────────────────────

/** ID of the agent message currently being streamed into */
let currentAgentMessageId: string | null = null;
let unlisteners: UnlistenFn[] = [];

/** Tracks whether an error was received before session:complete fires */
let hadRecentError = false;

/** Start listening for Tauri session events from the Rust backend. */
export async function startSessionListeners(): Promise<void> {
  stopSessionListeners();

  // Text output — accumulate token deltas into a single agent message.
  // With stream-json, each event is a token-level delta (not a full line),
  // so we concatenate directly without adding newlines.
  unlisteners.push(
    await listen<{ session_id: string; text: string }>(
      "session:output",
      (event) => {
        const { text } = event.payload;

        if (!currentAgentMessageId) {
          const id = crypto.randomUUID();
          currentAgentMessageId = id;
          messages.update((msgs) => [
            ...msgs,
            {
              id,
              role: "agent",
              content: text,
              timestamp: new Date().toISOString(),
            },
          ]);
        } else {
          const msgId = currentAgentMessageId;
          messages.update((msgs) =>
            msgs.map((m) =>
              m.id === msgId ? { ...m, content: m.content + text } : m,
            ),
          );
        }
      },
    ),
  );

  // Question detected — same accumulation, plus store suggested answers
  unlisteners.push(
    await listen<{
      session_id: string;
      text: string;
      suggested_answers: string[];
    }>("session:question", (event) => {
      const { text, suggested_answers } = event.payload;

      // Update suggested answers for the UI
      if (suggested_answers && suggested_answers.length > 0) {
        suggestedAnswers.set(suggested_answers);
      }

      if (!currentAgentMessageId) {
        const id = crypto.randomUUID();
        currentAgentMessageId = id;
        messages.update((msgs) => [
          ...msgs,
          {
            id,
            role: "agent",
            content: text,
            timestamp: new Date().toISOString(),
          },
        ]);
      } else {
        const msgId = currentAgentMessageId;
        messages.update((msgs) =>
          msgs.map((m) =>
            m.id === msgId ? { ...m, content: m.content + "\n" + text } : m,
          ),
        );
      }
    }),
  );

  // Streaming indicator (typing state)
  unlisteners.push(
    await listen<{ session_id: string; streaming: boolean }>(
      "session:streaming",
      (event) => {
        isAgentTyping.set(event.payload.streaming);

        if (!event.payload.streaming) {
          // Response boundary — finalize the current agent message
          currentAgentMessageId = null;
        }
      },
    ),
  );

  // Process exited
  unlisteners.push(
    await listen<{ session_id: string; text: string }>(
      "session:complete",
      () => {
        currentAgentMessageId = null;
        isAgentTyping.set(false);
        const crashed = hadRecentError;
        hadRecentError = false;
        currentSession.update((s) =>
          s
            ? { ...s, status: crashed ? ("error" as const) : ("ended" as const) }
            : s,
        );
      },
    ),
  );

  // Stderr / errors
  unlisteners.push(
    await listen<{ session_id: string; error: string }>(
      "session:error",
      (event) => {
        const { error } = event.payload;
        hadRecentError = true;
        messages.update((msgs) => [
          ...msgs,
          {
            id: crypto.randomUUID(),
            role: "system",
            content: `Error: ${error}`,
            timestamp: new Date().toISOString(),
          },
        ]);
      },
    ),
  );

  // Tool use — show as system message so user sees what Claude is doing
  unlisteners.push(
    await listen<{ session_id: string; tool: string; input_summary: string }>(
      "session:tool-use",
      (event) => {
        const { tool, input_summary } = event.payload;
        messages.update((msgs) => [
          ...msgs,
          {
            id: crypto.randomUUID(),
            role: "system",
            content: `Using ${tool}: ${input_summary}`,
            timestamp: new Date().toISOString(),
          },
        ]);
      },
    ),
  );

  // Permission request — show as system message with warning
  unlisteners.push(
    await listen<{ session_id: string; tool: string; description: string }>(
      "session:permission-request",
      (event) => {
        const { tool, description } = event.payload;
        messages.update((msgs) => [
          ...msgs,
          {
            id: crypto.randomUUID(),
            role: "system",
            content: `Permission requested: ${description}\nTo avoid prompts, configure --permission-mode in Settings or add --dangerously-skip-permissions to the CLI path.`,
            timestamp: new Date().toISOString(),
          },
        ]);
      },
    ),
  );
}

/** Remove all session event listeners. */
export function stopSessionListeners(): void {
  unlisteners.forEach((fn) => fn());
  unlisteners = [];
  currentAgentMessageId = null;
}

/** Add a user message to the local message list (call before sendMessage IPC). */
export function addUserMessage(content: string): void {
  suggestedAnswers.set([]); // Clear suggestions when user sends
  messages.update((msgs) => [
    ...msgs,
    {
      id: crypto.randomUUID(),
      role: "user",
      content,
      timestamp: new Date().toISOString(),
    },
  ]);
}
