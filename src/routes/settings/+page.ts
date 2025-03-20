import { get_core_settings } from "@/lib/utils";

export async function load() {
  const settings = await get_core_settings();
  return {
    settings,
  };
}
