import { invoke } from "@tauri-apps/api/core";
import type { PipelineState } from "$lib/stores/pipeline";
import type { AppSettings } from "$lib/stores/settings";

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
