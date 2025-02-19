import { invoke } from "@tauri-apps/api/core";

import type { Settings } from "$lib/types";

export async function load() {
  let settings: Settings = await invoke("get_settings_json");
  return {
    settings,
  };
}
