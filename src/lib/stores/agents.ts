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
  eco_mode: false,
  stop_protected: false,
  last_compact_save: null,
  orchestrator_checkpoint: null,
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

export const orchEcoMode = derived(orchestrationState, ($s) => $s.eco_mode);
export const orchStopProtected = derived(orchestrationState, ($s) => $s.stop_protected);

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

  // Session text output — parse for orchestration patterns in real time.
  // This provides immediate feedback before PROGRESS.md / EVENTS.jsonl
  // are written and polled.
  let textBuffer = "";
  unlisteners.push(
    await listen<{ session_id: string; text: string }>(
      "session:output",
      (event) => {
        textBuffer += event.payload.text;

        // Process complete lines
        let newlineIdx: number;
        while ((newlineIdx = textBuffer.indexOf("\n")) !== -1) {
          const line = textBuffer.slice(0, newlineIdx).trim();
          textBuffer = textBuffer.slice(newlineIdx + 1);

          if (!line) continue;

          const orchEvent = parseOrchestrationPattern(line);
          if (orchEvent) {
            orchestrationState.update((s) => {
              const updated = { ...s };

              // Set status to running on first detected event
              if (updated.status === "idle") {
                updated.status = "running";
              }

              updated.log = [
                ...updated.log,
                {
                  timestamp: new Date().toISOString(),
                  level: orchEvent.level,
                  source: orchEvent.source,
                  message: orchEvent.message,
                },
              ];
              return updated;
            });
          }
        }
      },
    ),
  );
}

// ── Orchestration pattern detection from session text ────────────────────────

interface OrchPattern {
  level: "system" | "agent" | "hook" | "error";
  source: string;
  message: string;
}

// Matches: "Launching coder-1 for P1.T1 — description (complexity: M, model: sonnet)"
const LAUNCH_RE = /^(?:Launching|Spawning)\s+([\w-]+)\s+for\s+(P\d+\.T\d+)/i;
// Matches: "Phase 1 coder agent completed" or "coder-1 completed successfully"
const COMPLETE_RE = /^(?:Phase\s+\d+\s+)?(\w[\w-]*)\s+(?:agent\s+)?completed/i;
// Matches: "Phase 1 PASSED review"
const REVIEW_RE = /^Phase\s+(\d+)\s+(PASSED|FAILED)\s+review/i;
// Matches: "moving to Phase 2" or "Phase 2: Name — starting"
const PHASE_RE = /(?:moving\s+to\s+Phase|starting\s+Phase|Phase)\s+(\d+)/i;

function parseOrchestrationPattern(line: string): OrchPattern | null {
  let m: RegExpMatchArray | null;

  m = line.match(LAUNCH_RE);
  if (m) {
    return {
      level: "agent",
      source: `agent:${m[1]}`,
      message: `Spawned for ${m[2]}`,
    };
  }

  m = line.match(COMPLETE_RE);
  if (m) {
    return {
      level: "agent",
      source: `agent:${m[1]}`,
      message: `Completed successfully`,
    };
  }

  m = line.match(REVIEW_RE);
  if (m) {
    return {
      level: "agent",
      source: "agent:reviewer",
      message: `Phase ${m[1]} review: ${m[2]}`,
    };
  }

  // Phase transition — only match if it looks like a transition announcement
  if (/(?:moving to|starting|beginning|entering)\s+Phase/i.test(line)) {
    m = line.match(PHASE_RE);
    if (m) {
      return {
        level: "system",
        source: "system",
        message: `Transitioning to Phase ${m[1]}`,
      };
    }
  }

  return null;
}

export function stopOrchestrationListeners(): void {
  unlisteners.forEach((fn) => fn());
  unlisteners = [];
}

// Re-export types for backward compatibility
export type { AgentInfo, TaskInfo, HookResultInfo, LogEntry, PhaseInfo };
export type { MailboxMessage } from "$lib/utils/ipc";
