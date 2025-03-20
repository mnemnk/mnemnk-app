import { invoke } from "@tauri-apps/api/core";

import type { CoreSettings, DailyStats, MnemnkEvent } from "./types";

const isEdge = typeof navigator !== "undefined" && navigator.userAgent?.includes("Edg");

export function mimgUrl(path: string): string {
  return isEdge ? `http://mimg.localhost/${path}` : `mimg://localhost/${path}`;
}

export function dateString(date: Date): string {
  return `${date.getFullYear()}${(date.getMonth() + 1)
    .toString()
    .padStart(2, "0")}${date.getDate().toString().padStart(2, "0")}`;
}

export function formatDate(date: Date): string {
  return `${date.getFullYear()}/${(date.getMonth() + 1)
    .toString()
    .padStart(2, "0")}/${date.getDate().toString().padStart(2, "0")}`;
}

export function formatTime(date: Date): string {
  return `${date.getHours().toString().padStart(2, "0")}:${date
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;
}

// app

export async function exit_app(): Promise<void> {
  await invoke("exit_app_cmd");
}

// events

export async function find_events_by_ymd(
  year: number,
  month: number,
  day: number,
): Promise<MnemnkEvent[]> {
  return await invoke("find_events_by_ymd_cmd", { year, month, day });
}

export async function daily_stats(): Promise<DailyStats[]> {
  return await invoke("daily_stats_cmd");
}

// search

export async function search_events(query: string): Promise<MnemnkEvent[]> {
  return await invoke("search_events_cmd", { query });
}

// settings

export async function get_core_settings(): Promise<CoreSettings> {
  return await invoke("get_core_settings_cmd");
}

export async function set_core_settings(new_settings: CoreSettings): Promise<void> {
  await invoke("set_core_settings_cmd", { new_settings });
}
