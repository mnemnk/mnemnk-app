import type { SAgentDefinitions } from "$lib/types";

import { deserializeAgentFlow, getAgentFlows, getAgentDefs } from "@/lib/agent";
import { getAgentGlobalConfigs, getDailyStats, getCoreSettings } from "@/lib/utils";

// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export async function load() {
  const dailyStats = await getDailyStats();
  const coreSettings = await getCoreSettings();

  const agentDefs: SAgentDefinitions = await getAgentDefs();
  const agentGlobalConfigs = await getAgentGlobalConfigs();

  const sAgentFlows = await getAgentFlows();
  const agentFlows = Object.fromEntries(
    Object.entries(sAgentFlows).map(([key, flow]) => [key, deserializeAgentFlow(flow, agentDefs)]),
  );

  return {
    dailyStats,
    coreSettings,
    agentDefs,
    agentGlobalConfigs,
    agentFlows,
  };
}
