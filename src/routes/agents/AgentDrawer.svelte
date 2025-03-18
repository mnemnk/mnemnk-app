<script lang="ts">
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";

  import { addAgentNode, updateAgentFlow } from "@/lib/agent";
  import type { SAgentConfigs, AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  type Props = NodeProps & {
    nodes: Writable<AgentFlowNode[]>;
    edges: Writable<AgentFlowEdge[]>;
    flow_index: number;
    agent_configs: SAgentConfigs;
  };

  const { nodes, edges, flow_index, agent_configs }: Props = $props();

  const agent_names = Object.keys(agent_configs).sort();

  function addAgent(agent_name: string) {
    addAgentNode(agent_name, nodes, agent_configs);
  }

  // function addBoard() {
  //   addBoardNode(nodes, agent_configs);
  // }

  // function addDatabase() {
  //   addDatabaseNode(nodes, agent_configs);
  // }

  async function update() {
    await updateAgentFlow(nodes, edges, flow_index, agent_configs);
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
  <!-- <Button class="w-full mb-4" color="blue" outline onclick={addBoard}>Board</Button>
  <Button class="w-full mb-4" color="blue" outline onclick={addDatabase}>Database</Button> -->
  {#each agent_names as agent_name}
    <div class="mb-4">
      <Button
        class="w-full"
        color={agent_name.startsWith("$") ? "blue" : "primary"}
        outline
        onclick={() => addAgent(agent_name)}>{agent_configs[agent_name].title ?? agent_name}</Button
      >
    </div>
  {/each}
</Drawer>
