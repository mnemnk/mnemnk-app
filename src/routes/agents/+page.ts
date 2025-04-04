import type { SAgentDefinitions } from "$lib/types";

import { deserializeAgentFlow, getAgentFlows, getAgentDefs } from "@/lib/agent";

export async function load() {
  const agentDefs: SAgentDefinitions = await getAgentDefs();

  const sAgentFlows = await getAgentFlows();
  const agentFlows = Object.fromEntries(
    Object.entries(sAgentFlows).map(([key, flow]) => [key, deserializeAgentFlow(flow, agentDefs)]),
  );

  return {
    agentDefs,
    agentFlows,
  };
}
