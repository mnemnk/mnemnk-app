import type { SAgentConfigs } from "$lib/types";

import { getAgentFlows, getAgentConfigs } from "@/lib/agent";

export async function load() {
  const agent_configs: SAgentConfigs = await getAgentConfigs();

  const agent_flows = await getAgentFlows();
  if (agent_flows.length === 0) {
    agent_flows.push({ nodes: [], edges: [] });
  }

  return {
    agent_configs,
    agent_flows,
  };
}
