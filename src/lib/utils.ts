import { invoke } from "@tauri-apps/api/core";

import type { AgentCatalogEntry, CoreSettings, Settings } from "./types";

const isEdge = typeof navigator !== "undefined" && navigator.userAgent?.includes("Edg");

export function mimgUrl(path: string): string {
  return isEdge ? `http://mimg.localhost/${path}` : `mimg://localhost/${path}`;
}

export async function find_events_by_ymd(year: number, month: number, day: number) {
  return await invoke("find_events_by_ymd", { year, month, day });
}

export async function index_year(year: number): Promise<[string, number][]> {
  let results = await invoke("index_year", { year });
  let ret: [string, number][] = [];
  for (let i = 0; i < results.dates.length; i++) {
    let date = results.dates[i];
    let num_event = results.num_events[i];
    ret.push([date, num_event]);
  }
  return ret;
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

// agent

export async function get_agent_catalog(): Promise<AgentCatalogEntry[]> {
  return await invoke("get_agent_catalog_cmd");
}

export async function start_agent(agent: string): Promise<void> {
  await invoke("start_agent_cmd", { agent });
}

export async function stop_agent(agent: string): Promise<void> {
  await invoke("stop_agent_cmd", { agent });
}

// settings

export async function get_settings_filepath(): Promise<string> {
  return await invoke("get_settings_filepath");
}

export async function get_settings_json(): Promise<Settings> {
  return await invoke("get_settings_json");
}

export async function set_core_settings(new_settings: CoreSettings): Promise<void> {
  await invoke("set_core_settings_cmd", { new_settings });
}

export async function set_agent_enabled(agent: string, enabled: boolean): Promise<void> {
  await invoke("set_agent_enabled_cmd", { agent, enabled });
}
