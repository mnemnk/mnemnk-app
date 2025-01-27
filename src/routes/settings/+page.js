import { invoke } from "@tauri-apps/api/core";

export async function load({ url }) {
  let settings = await invoke("get_settings_json");
  return {
    settings,
  };
}
