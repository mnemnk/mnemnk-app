import { getAgentDefs } from "@/lib/agent";
import { getCoreSettings, getAgentGlobalConfigs } from "@/lib/utils";

export async function load() {
  const settings = await getCoreSettings();
  const agentDefs = await getAgentDefs();
  const agentConfigs = await getAgentGlobalConfigs();

  return {
    settings,
    agentDefs,
    agentConfigs,
  };
}
