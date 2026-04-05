import { writable, derived, get } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { currentSession } from "./session";

export interface PipelineState {
  gddExists: boolean;
  tddExists: boolean;
  workflowExists: boolean;
  progressExists: boolean;
  currentPhase: string;
}

export interface AssetCounts {
  scripts: number;
  tests: number;
  prefabs: number;
  configs: number;
}

// ── Phase definitions ─────────────────────────────────────────────────────

export interface PipelinePhase {
  id: string;
  key: string;
  label: string;
  command: string;
  status: "done" | "active" | "ready" | "locked";
}

export const PHASE_ORDER = [
  { id: "game_idea", key: "idea", label: "Idea", command: "/game-idea" },
  { id: "architect", key: "architecture", label: "Architecture", command: "/architect" },
  { id: "plan_workflow", key: "planning", label: "Planning", command: "/plan-workflow" },
  { id: "orchestrate", key: "building", label: "Building", command: "/orchestrate" },
  { id: "complete", key: "complete", label: "Complete", command: "" },
] as const;

// ── Core state ────────────────────────────────────────────────────────────

export const pipelineState = writable<PipelineState>({
  gddExists: false,
  tddExists: false,
  workflowExists: false,
  progressExists: false,
  currentPhase: "none",
});

/** Event fired when a phase artifact is newly detected */
export const phaseJustCompleted = writable<string | null>(null);

// ── Derived: phases with computed status ──────────────────────────────────

export const phases = derived(
  [pipelineState, currentSession],
  ([$ps, $session]) => {
    const activePhaseId = $session?.status === "active" ? $session.phase : null;

    return PHASE_ORDER.map((def, i) => {
      const done = isPhaseComplete(def.id, $ps);
      const prevDone = i === 0 || isPhaseComplete(PHASE_ORDER[i - 1].id, $ps);

      let status: PipelinePhase["status"];
      if (done) {
        status = "done";
      } else if (activePhaseId === def.id) {
        status = "active";
      } else if (prevDone && !done) {
        status = "ready";
      } else {
        status = "locked";
      }

      return { ...def, status } as PipelinePhase;
    });
  },
);

/** The next phase that can be started (first "ready" phase) */
export const nextPhase = derived(phases, ($phases) =>
  $phases.find((p) => p.status === "ready") ?? null,
);

/** Overall pipeline progress as 0-1 ratio */
export const pipelineProgress = derived(phases, ($phases) => {
  const doneCount = $phases.filter((p) => p.status === "done").length;
  return doneCount / $phases.length;
});

// ── Helpers ───────────────────────────────────────────────────────────────

function isPhaseComplete(phaseId: string, ps: PipelineState): boolean {
  switch (phaseId) {
    case "game_idea": return ps.gddExists;
    case "architect": return ps.tddExists;
    case "plan_workflow": return ps.workflowExists;
    case "orchestrate": return ps.progressExists && ps.currentPhase === "complete";
    case "complete": return ps.progressExists && ps.currentPhase === "complete";
    default: return false;
  }
}

/** Returns the artifact file that a phase produces */
export function phaseArtifact(phaseId: string): string | null {
  switch (phaseId) {
    case "game_idea": return "GDD.md";
    case "architect": return "TDD.md";
    case "plan_workflow": return "WORKFLOW.md";
    case "orchestrate": return "PROGRESS.md";
    default: return null;
  }
}

// ── File watcher integration ──────────────────────────────────────────────

let watcherUnlisten: UnlistenFn | null = null;

/** Listen for doc changes from the Rust file watcher and update pipeline state. */
export async function startPipelineWatcher(): Promise<void> {
  stopPipelineWatcher();

  watcherUnlisten = await listen<{ file: string; exists: boolean }>(
    "docs-changed",
    (event) => {
      const { file, exists } = event.payload;
      const prev = get(pipelineState);

      const updates: Partial<PipelineState> = {};
      if (file.includes("GDD.md")) updates.gddExists = exists;
      if (file.includes("TDD.md")) updates.tddExists = exists;
      if (file.includes("WORKFLOW.md")) updates.workflowExists = exists;
      if (file.includes("PROGRESS.md")) updates.progressExists = exists;

      if (Object.keys(updates).length > 0) {
        // Detect newly completed phases
        if (updates.gddExists && !prev.gddExists) phaseJustCompleted.set("game_idea");
        if (updates.tddExists && !prev.tddExists) phaseJustCompleted.set("architect");
        if (updates.workflowExists && !prev.workflowExists) phaseJustCompleted.set("plan_workflow");

        pipelineState.update((s) => ({ ...s, ...updates }));
      }
    },
  );
}

export function stopPipelineWatcher(): void {
  watcherUnlisten?.();
  watcherUnlisten = null;
}
