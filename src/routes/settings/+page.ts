import { getAgentDefs } from "@/lib/agent";
import { get_core_settings, getAgentConfigs } from "@/lib/utils";

export async function load() {
  const settings = await get_core_settings();
  const agentDefs = await getAgentDefs();
  const agentConfigs = await getAgentConfigs();

  return {
    settings,
    agentDefs,
    agentConfigs,
  };
}
