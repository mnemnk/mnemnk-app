<script lang="ts">
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";

  import { addAgentNode, addBoardNode, addDatabaseNode, updateAgentFlow } from "@/lib/agent";
  import type { AgentCatalog, AgentSetting, AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  type Props = NodeProps & {
    nodes: Writable<AgentFlowNode[]>;
    edges: Writable<AgentFlowEdge[]>;
    flow_index: number;
    catalog: AgentCatalog;
    settings: Record<string, AgentSetting>;
  };

  const { nodes, edges, flow_index, catalog, settings }: Props = $props();

  function addAgent(agent_name: string) {
    addAgentNode(agent_name, nodes, settings);
  }

  function addBoard() {
    addBoardNode(nodes, settings);
  }

  function addDatabase() {
    addDatabaseNode(nodes, settings);
  }

  async function update() {
    await updateAgentFlow(nodes, edges, flow_index);
  }
</script>

<Drawer
  activateClickOutside={false}
  backdrop={false}
  hidden={false}
  placement="right"
  class="w-200"
>
  <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={update}>Update</GradientButton>
  <Button class="w-full mb-4" color="blue" outline onclick={addBoard}>Board</Button>
  <Button class="w-full mb-4" color="blue" outline onclick={addDatabase}>Database</Button>
  {#each catalog as agent}
    <div class="mb-4">
      <Button class="w-full" outline onclick={() => addAgent(agent.name)}>{agent.name}</Button>
    </div>
  {/each}
</Drawer>
