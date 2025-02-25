import type { Settings } from "$lib/types";

import { get_core_settings } from "@/lib/utils";

export async function load() {
  let settings: Settings = await get_core_settings();
  return {
    settings,
  };
}
