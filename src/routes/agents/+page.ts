import type { AgentCatalogEntry, AgentSettings } from "$lib/types";

import { get_agent_catalog, get_agent_flows, get_agent_settings } from "@/lib/agent";

export async function load() {
  let catalog: AgentCatalogEntry[] = await get_agent_catalog();
  catalog = catalog.sort((a, b) => a.name.localeCompare(b.name));

  const settings: Record<string, AgentSettings> = await get_agent_settings();

  const agent_flows = await get_agent_flows();
  if (agent_flows.length === 0) {
    agent_flows.push({ nodes: [], edges: [] });
  }

  return {
    agent_flows,
    catalog,
    settings,
  };
}
