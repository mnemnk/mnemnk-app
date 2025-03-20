<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Drawer, GradientButton } from "flowbite-svelte";

  import {
    addAgentNode,
    deserializeAgentFlow,
    readAgentFlow,
    serializeAgentFlow,
    updateAgentFlow,
  } from "@/lib/agent";
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

  async function updateFlow() {
    await updateAgentFlow(nodes, edges, flow_index, agent_configs);
  }

  function exportFlow() {
    const flow = serializeAgentFlow(get(nodes), get(edges), agent_configs);
    const jsonStr = JSON.stringify(flow, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "flow.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function importFlow() {
    const file = await open({ multiple: false, filter: "json" });
    if (!file) return;
    const sflow = await readAgentFlow(file);
    if (!sflow.nodes || !sflow.edges) return;
    const flow = deserializeAgentFlow(sflow, agent_configs);
    nodes.set(flow.nodes);
    edges.set(flow.edges);
  }
</script>

<Drawer
  activateClickOutside={false}
  backdrop={false}
  hidden={false}
  placement="right"
  class="w-200"
>
  <GradientButton color="pinkToOrange" class="w-full mb-4" onclick={updateFlow}
    >Update</GradientButton
  >
  <GradientButton color="purpleToBlue" class="w-full mb-4" onclick={exportFlow}
    >Export</GradientButton
  >
  <GradientButton color="purpleToPink" class="w-full mb-4" onclick={importFlow}
    >Import</GradientButton
  >
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
