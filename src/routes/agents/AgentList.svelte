<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import { Button } from "flowbite-svelte";

  import type { SAgentDefinitions, AgentFlowNode } from "@/lib/types";

  type Props = NodeProps & {
    nodes: AgentFlowNode[];
    agent_defs: SAgentDefinitions;
    onAddAgent: (agent_name: string) => Promise<void>;
  };

  const { agent_defs, onAddAgent }: Props = $props();

  const agent_names = Object.keys(agent_defs).sort();
</script>

<div class="w-200">
  {#each agent_names as agent_name}
    <Button outline class="w-full mb-2" onclick={() => onAddAgent(agent_name)}
      >{agent_defs[agent_name].title ?? agent_name}</Button
    >
  {/each}
</div>
