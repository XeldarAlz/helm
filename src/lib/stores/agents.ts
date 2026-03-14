import { writable, derived } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  OrchestrationState,
  AgentInfo,
  TaskInfo,
  HookResultInfo,
  LogEntry,
  PhaseInfo,
} from "$lib/utils/ipc";

// ── Orchestration State ──────────────────────────────────────────────────────

export const orchestrationState = writable<OrchestrationState>({
  status: "idle",
  current_phase: 0,
  total_phases: 0,
  phase_name: "",
  started_at: null,
  phases: [],
  agents: [],
  tasks: [],
  hooks: [],
  log: [],
});

// Derived stores for convenient access
export const orchStatus = derived(orchestrationState, ($s) => $s.status);
export const orchPhases = derived(orchestrationState, ($s) => $s.phases);
export const orchAgents = derived(orchestrationState, ($s) => $s.agents);
export const orchTasks = derived(orchestrationState, ($s) => $s.tasks);
export const orchHooks = derived(orchestrationState, ($s) => $s.hooks);
export const orchLog = derived(orchestrationState, ($s) => $s.log);

export const orchProgress = derived(orchestrationState, ($s) => {
  if ($s.total_phases === 0) return 0;
  const donePhases = $s.phases.filter((p) => p.status === "done").length;
  return Math.round((donePhases / $s.total_phases) * 100);
});

export const isOrchestrating = derived(
  orchestrationState,
  ($s) => $s.status === "running" || $s.status === "paused",
);

// ── Log filtering ────────────────────────────────────────────────────────────

export type LogFilter = "all" | "agents" | "hooks" | "errors";
export const logFilter = writable<LogFilter>("all");

export const filteredLog = derived(
  [orchLog, logFilter],
  ([$log, $filter]) => {
    if ($filter === "all") return $log;
    if ($filter === "agents") return $log.filter((e) => e.level === "agent");
    if ($filter === "hooks") return $log.filter((e) => e.level === "hook");
    if ($filter === "errors") return $log.filter((e) => e.level === "error");
    return $log;
  },
);

// ── Event listeners ──────────────────────────────────────────────────────────

let unlisteners: UnlistenFn[] = [];

export async function startOrchestrationListeners(): Promise<void> {
  stopOrchestrationListeners();

  // Agent spawned
  unlisteners.push(
    await listen<{ session_id: string; agent_type: string; task: string }>(
      "orchestration:agent-spawned",
      (event) => {
        const { agent_type, task } = event.payload;
        orchestrationState.update((s) => ({
          ...s,
          log: [
            ...s.log,
            {
              timestamp: new Date().toISOString(),
              level: "agent" as const,
              source: `agent:${agent_type}`,
              message: `Spawned: ${task}`,
            },
          ],
        }));
      },
    ),
  );

  // Agent completed
  unlisteners.push(
    await listen<{ session_id: string; agent_type: string; task: string }>(
      "orchestration:agent-completed",
      (event) => {
        const { agent_type, task } = event.payload;
        orchestrationState.update((s) => ({
          ...s,
          log: [
            ...s.log,
            {
              timestamp: new Date().toISOString(),
              level: "agent" as const,
              source: `agent:${agent_type}`,
              message: `Completed: ${task}`,
            },
          ],
        }));
      },
    ),
  );

  // Phase change
  unlisteners.push(
    await listen<{ session_id: string; phase: number; name: string }>(
      "orchestration:phase-change",
      (event) => {
        const { phase, name } = event.payload;
        orchestrationState.update((s) => ({
          ...s,
          current_phase: phase,
          phase_name: name,
          log: [
            ...s.log,
            {
              timestamp: new Date().toISOString(),
              level: "system" as const,
              source: "system",
              message: `Phase ${phase}: ${name}`,
            },
          ],
        }));
      },
    ),
  );
}

export function stopOrchestrationListeners(): void {
  unlisteners.forEach((fn) => fn());
  unlisteners = [];
}

// Re-export types for backward compatibility
export type { AgentInfo, TaskInfo, HookResultInfo, LogEntry, PhaseInfo };
