import { invoke } from "@tauri-apps/api/core";
import type { PipelineState, AssetCounts } from "$lib/stores/pipeline";
import type { AppSettings } from "$lib/stores/settings";
import type { SessionSummary } from "$lib/stores/session";

export async function createSession(phase: string): Promise<string> {
  return invoke("create_session", { phase });
}

export async function sendMessage(sessionId: string, message: string): Promise<void> {
  return invoke("send_message", { sessionId, message });
}

export async function endSession(sessionId: string): Promise<void> {
  return invoke("end_session", { sessionId });
}

export async function getPipelineState(): Promise<PipelineState> {
  return invoke("get_pipeline_state");
}

export async function readDocument(name: string): Promise<string> {
  return invoke("read_document", { name });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function getAssetCounts(): Promise<AssetCounts> {
  return invoke("get_asset_counts");
}

export async function listSessions(): Promise<SessionSummary[]> {
  return invoke("list_sessions");
}

export async function checkCli(): Promise<boolean> {
  return invoke("check_cli");
}

export async function getSessionMessages(sessionId: string): Promise<{ id: string; role: string; content: string; timestamp: string }[]> {
  return invoke("get_session_messages", { sessionId });
}

export async function saveTranscript(sessionId: string): Promise<string> {
  return invoke("save_transcript", { sessionId });
}

// ── Phase 5: File Watcher ───────────────────────────────────────────────────

export async function startWatching(): Promise<void> {
  return invoke("start_watching");
}

export async function stopWatching(): Promise<void> {
  return invoke("stop_watching");
}

// ── Phase 5: Code Browser ───────────────────────────────────────────────────

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileNode[];
  extension: string | null;
  size: number | null;
}

export async function getFileTree(root?: string): Promise<FileNode[]> {
  return invoke("get_file_tree", { root: root ?? null });
}

export async function readProjectFile(path: string): Promise<string> {
  return invoke("read_project_file", { path });
}

// ── Phase 6: Orchestration ──────────────────────────────────────────────────

export interface OrchestrationState {
  status: "idle" | "running" | "paused" | "stopped" | "completed" | "failed";
  current_phase: number;
  total_phases: number;
  phase_name: string;
  started_at: string | null;
  phases: PhaseInfo[];
  agents: AgentInfo[];
  tasks: TaskInfo[];
  hooks: HookResultInfo[];
  log: LogEntry[];
}

export interface PhaseInfo {
  number: number;
  name: string;
  status: "done" | "active" | "pending";
}

export interface AgentInfo {
  id: string;
  agent_type: "coder" | "tester" | "reviewer" | "unity_setup" | "committer";
  status: "idle" | "running" | "reviewing" | "passed" | "failed";
  current_task: string | null;
  progress: number;
}

export interface TaskInfo {
  id: string;
  title: string;
  status: "pending" | "working" | "review" | "done" | "failed";
  agent: string | null;
  complexity: string;
}

export interface HookResultInfo {
  name: string;
  last_run: string | null;
  status: "passed" | "warning" | "failed" | "running";
}

export interface LogEntry {
  timestamp: string;
  level: "system" | "agent" | "hook" | "error";
  source: string;
  message: string;
}

export async function getOrchestrationState(): Promise<OrchestrationState> {
  return invoke("get_orchestration_state");
}

export async function sendOrchestrationCommand(command: "pause" | "resume" | "stop"): Promise<void> {
  return invoke("send_orchestration_command", { command });
}

// ── Phase 7: Git ────────────────────────────────────────────────────────────

export interface GitCommit {
  hash: string;
  short_hash: string;
  author: string;
  date: string;
  message: string;
  files_changed: number;
  insertions: number;
  deletions: number;
}

export interface GitBranch {
  name: string;
  is_current: boolean;
  last_commit: string;
}

export interface GitDiffFile {
  path: string;
  status: string;
  insertions: number;
  deletions: number;
}

export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  staged: { path: string; status: string }[];
  unstaged: { path: string; status: string }[];
  untracked: string[];
}

export async function getGitLog(count?: number): Promise<GitCommit[]> {
  return invoke("get_git_log", { count: count ?? null });
}

export async function getGitBranches(): Promise<GitBranch[]> {
  return invoke("get_git_branches");
}

export async function getGitStatus(): Promise<GitStatus> {
  return invoke("get_git_status");
}

export async function getGitDiff(commitHash: string): Promise<GitDiffFile[]> {
  return invoke("get_git_diff", { commitHash });
}
