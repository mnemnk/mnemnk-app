import type { SAgentDefinitions } from "$lib/types";

import { getAgentFlows, getAgentDefs } from "@/lib/agent";

export async function load() {
  const agent_defs: SAgentDefinitions = await getAgentDefs();

  let agent_flows = await getAgentFlows();

  return {
    agent_defs,
    agent_flows,
  };
}
