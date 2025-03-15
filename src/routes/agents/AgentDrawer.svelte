<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";

  import {
    newAgentFlowBoardNode,
    newAgentFlowNode,
    save_agent_flow,
    serializeAgentFlow,
  } from "@/lib/agent";
  import type { AgentCatalog, AgentSetting, AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  type Props = NodeProps & {
    nodes: Writable<AgentFlowNode[]>;
    edges: Writable<AgentFlowEdge[]>;
    flow_index: number;
    catalog: AgentCatalog;
    settings: Record<string, AgentSetting>;
  };

  const { nodes, edges, flow_index, catalog, settings }: Props = $props();

  function addAgentNode(agent_name: string) {
    const new_node = newAgentFlowNode(agent_name, settings);
    nodes.update((nodes) => {
      return [...nodes, new_node];
    });
  }

  function addBoardNode() {
    const new_node = newAgentFlowBoardNode(settings);
    nodes.update((nodes) => {
      return [...nodes, new_node];
    });
  }

  async function updateAgentFlow() {
    const flow = serializeAgentFlow(get(nodes), get(edges));
    await save_agent_flow(flow, flow_index);
  }
</script>

<Drawer
  activateClickOutside={false}
  backdrop={false}
  hidden={false}
  placement="right"
  class="w-200"
>
  <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={updateAgentFlow}>
    Update
  </GradientButton>
  <Button class="w-full mb-4" color="blue" outline onclick={() => addBoardNode()}>Board</Button>
  {#each catalog as agent}
    <div class="mb-4">
      <Button class="w-full" outline onclick={() => addAgentNode(agent.name)}>{agent.name}</Button>
    </div>
  {/each}
</Drawer>
