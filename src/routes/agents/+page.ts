import type { SAgentDefinitions } from "$lib/types";

import { getAgentFlows, getAgentDefs } from "@/lib/agent";

export async function load() {
  const agent_defs: SAgentDefinitions = await getAgentDefs();

  const agent_flows = await getAgentFlows();
  if (agent_flows.length === 0) {
    agent_flows.push({ nodes: [], edges: [] });
  }

  return {
    agent_defs,
    agent_flows,
  };
}
