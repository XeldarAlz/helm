import { writable } from "svelte/store";

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

export const currentSession = writable<Session | null>(null);
export const messages = writable<Message[]>([]);
export const isAgentTyping = writable(false);
