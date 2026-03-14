import { writable } from "svelte/store";

export interface AgentState {
  id: string;
  type: "coder" | "tester" | "reviewer" | "unity" | "committer";
  status: "idle" | "running" | "reviewing" | "passed" | "failed";
  taskId?: string;
  progress: number;
}

export interface TaskState {
  id: string;
  title: string;
  status: "pending" | "working" | "review" | "done" | "failed";
  agentType?: string;
  complexity: "S" | "M" | "L";
}

export const agents = writable<AgentState[]>([]);
export const tasks = writable<TaskState[]>([]);
export const currentOrchPhase = writable(0);
