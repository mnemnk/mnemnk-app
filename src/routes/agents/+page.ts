import type { AgentCatalogEntry, Settings } from "$lib/types";

import { get_agent_catalog, get_settings_json } from "@/lib/utils";

export async function load() {
  let catalog: AgentCatalogEntry[] = await get_agent_catalog();
  catalog = catalog.sort((a, b) => a.name.localeCompare(b.name));

  let settings: Settings = await get_settings_json();

  for (let agent of catalog) {
    if (!settings.agents[agent.name]) {
      settings.agents[agent.name] = {
        enabled: false,
        config: null,
      };
    } else if (settings.agents[agent.name].enabled === null) {
      settings.agents[agent.name].enabled = false;
    }
  }

  return {
    catalog,
    settings,
  };
}
