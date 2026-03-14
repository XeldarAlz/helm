import { writable } from "svelte/store";

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

export const pipelineState = writable<PipelineState>({
  gddExists: false,
  tddExists: false,
  workflowExists: false,
  progressExists: false,
  currentPhase: "none",
});
