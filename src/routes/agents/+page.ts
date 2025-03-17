import type { SAgentConfigs } from "$lib/types";

import { get_agent_flows, get_agent_configs } from "@/lib/agent";

export async function load() {
  const agent_configs: SAgentConfigs = await get_agent_configs();

  const agent_flows = await get_agent_flows();
  if (agent_flows.length === 0) {
    agent_flows.push({ nodes: [], edges: [] });
  }

  return {
    agent_configs,
    agent_flows,
  };
}
