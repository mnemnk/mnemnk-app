<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { Edge, NodeProps } from "@xyflow/svelte";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";
  import { nanoid } from "nanoid";

  import { deserializeAgentFlowNode, save_agent_flow, serializeAgentFlow } from "@/lib/agent";
  import type { AgentCatalogEntry, AgentSettings, AgentFlowNode } from "@/lib/types";

  type Props = NodeProps & {
    nodes: Writable<AgentFlowNode[]>;
    edges: Writable<Edge[]>;
    flow_index: number;
    catalog: AgentCatalogEntry[];
    settings: Record<string, AgentSettings>;
  };

  const { nodes, edges, flow_index, catalog, settings }: Props = $props();

  function addAgentNode(agent_name: string) {
    const id = nanoid();
    const default_config = settings[agent_name]?.default_config || {};
    const node_data = {
      id,
      name: agent_name,
      enabled: true,
      config: {
        ...default_config,
      },
      x: Math.random() * 1000,
      y: Math.random() * 1000,
    };
    nodes.update((nodes) => {
      return [...nodes, deserializeAgentFlowNode(node_data, settings)];
    });
  }

  function addBoardNode() {
    const id = nanoid();
    const node_data = {
      id,
      name: "$board",
      enabled: true,
      config: {
        board_name: "",
        persistent: false,
      },
      x: Math.random() * 1000,
      y: Math.random() * 1000,
    };
    nodes.update((nodes) => {
      return [...nodes, deserializeAgentFlowNode(node_data, settings)];
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
