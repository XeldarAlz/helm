import { writable } from "svelte/store";

export interface AppSettings {
  projectDir: string;
  claudeCliPath: string;
  maxConcurrentAgents: number;
  agentModels: {
    coder: string;
    tester: string;
    reviewer: string;
    unitySetup: string;
    committer: string;
  };
  fontSize: number;
  sidebarCollapsed: boolean;
}

export const settings = writable<AppSettings>({
  projectDir: "",
  claudeCliPath: "claude",
  maxConcurrentAgents: 3,
  agentModels: {
    coder: "opus",
    tester: "sonnet",
    reviewer: "opus",
    unitySetup: "sonnet",
    committer: "haiku",
  },
  fontSize: 13,
  sidebarCollapsed: false,
});
