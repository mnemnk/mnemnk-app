import { invoke } from "@tauri-apps/api/core";

const isEdge = navigator.userAgent.includes("Edg");

export function mimgUrl(path: string): string {
  return isEdge ? `http://mimg.localhost/${path}` : `mimg://localhost/${path}`;
}

export async function find_events_by_ymd(year: number, month: number, day: number) {
  return await invoke("find_events_by_ymd", { year, month, day });
}

export async function index_year(year: number) {
  let results = await invoke("index_year", { year });
  let ret = [];
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
