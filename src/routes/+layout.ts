import { daily_stats } from "@/lib/utils";
import { get_core_settings } from "@/lib/utils";

// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export async function load() {
  const stats = await daily_stats();
  const settings = await get_core_settings();
  return {
    daily_stats: stats,
    settings,
  };
}
