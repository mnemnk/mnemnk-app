<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { SvelteFlow, Controls, type NodeTypes, useSvelteFlow, MiniMap } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import {
    Button,
    ButtonGroup,
    Dropdown,
    DropdownItem,
    GradientButton,
    Modal,
    Navbar,
    NavLi,
    NavUl,
  } from "flowbite-svelte";
  import { ChevronDownOutline, PauseOutline, PlayOutline } from "flowbite-svelte-icons";
  import hotkeys from "hotkeys-js";

  import {
    addAgentFlowEdge,
    addAgentFlowNode,
    removeAgentFlowEdge,
    removeAgentFlowNode,
    deserializeAgentFlow,
    deserializeAgentFlowNode,
    importAgentFlow,
    newAgentFlow,
    newAgentFlowNode,
    saveAgentFlow,
    serializeAgentFlow,
    serializeAgentFlowEdge,
    setAgentDefinitionsContext,
    startAgent,
    stopAgent,
  } from "@/lib/agent";
  import type { AgentFlowNode, AgentFlowEdge } from "@/lib/types";

  import AgentMegaMenu from "./AgentMegaMenu.svelte";
  import AgentNode from "./AgentNode.svelte";
  import FlowMegaMenu from "./FlowMegaMenu.svelte";

  let { data } = $props();

  const { screenToFlowPosition, updateNodeData } = $derived(useSvelteFlow());
  setAgentDefinitionsContext(data.agent_defs);

  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  const agent_defs = data.agent_defs;

  let nodes = $state.raw<AgentFlowNode[]>([]);
  let edges = $state.raw<AgentFlowEdge[]>([]);
  let flows = $state(
    Object.fromEntries(
      Object.entries(data.agent_flows).map(([key, flow]) => [
        key,
        deserializeAgentFlow(flow, data.agent_defs),
      ]),
    ),
  );
  let flowName = $state("main" in flows ? "main" : Object.keys(flows)[0] || "");

  $effect(() => {
    if (flowName in flows) {
      nodes = flows[flowName].nodes;
      edges = flows[flowName].edges;
    }
  });

  async function checkNodeChange(nodes: AgentFlowNode[]) {
    const nodeIds = new Set(nodes.map((node) => node.id));

    const deletedNodes = flows[flowName].nodes.filter((node) => !nodeIds.has(node.id));
    for (const node of deletedNodes) {
      await removeAgentFlowNode(flowName, node.id);
      flows[flowName].nodes = flows[flowName].nodes.filter((n) => n.id !== node.id);
    }
  }

  async function checkEdgeChange(edges: AgentFlowEdge[]) {
    const edgeIds = new Set(edges.map((edge) => edge.id));

    const deletedEdges = flows[flowName].edges.filter((edge) => !edgeIds.has(edge.id));
    for (const edge of deletedEdges) {
      await removeAgentFlowEdge(flowName, edge.id);
      flows[flowName].edges = flows[flowName].edges.filter((e) => e.id !== edge.id);
    }

    const addedEdges = edges.filter((edge) => !flows[flowName].edges.some((e) => e.id === edge.id));
    for (const edge of addedEdges) {
      await addAgentFlowEdge(flowName, serializeAgentFlowEdge(edge));
      flows[flowName].edges.push(edge);
    }
  }

  $effect(() => {
    checkNodeChange(nodes);
    checkEdgeChange(edges);
  });

  // shortcuts

  let openAgent = $state(false);
  const key_open_agent = "a";

  let openFlow = $state(false);
  const key_open_flow = "f";

  $effect(() => {
    hotkeys("ctrl+s", (event) => {
      event.preventDefault();
      onSaveFlow();
    });

    hotkeys(key_open_agent, () => {
      openAgent = !openAgent;
    });

    hotkeys(key_open_flow, () => {
      openFlow = !openFlow;
    });

    return () => {
      hotkeys.unbind("ctrl+s");
      hotkeys.unbind(key_open_agent);
      hotkeys.unbind(key_open_flow);
    };
  });

  // New Flow

  let newFlowModal = $state(false);
  let newFlowName = $state("");

  async function onNewFlow() {
    newFlowName = "";
    newFlowModal = true;
  }

  async function createNewFlow() {
    newFlowModal = false;
    if (!newFlowName) return;
    const flow = await newAgentFlow(newFlowName);
    if (!flow) return;
    flows[flow.name] = deserializeAgentFlow(flow, agent_defs);
    flowName = flow.name;
  }

  async function onSaveFlow() {
    if (flowName in flows) {
      const flow = serializeAgentFlow(nodes, edges, flowName, data.agent_defs);
      await saveAgentFlow(flow);
      flows[flowName] = deserializeAgentFlow(flow, data.agent_defs);
    }
  }

  function onExportFlow() {
    const flow = serializeAgentFlow(nodes, edges, flowName, agent_defs);
    const jsonStr = JSON.stringify(flow, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = flowName + ".json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function onImportFlow() {
    const file = await open({ multiple: false, filter: "json" });
    if (!file) return;
    const sflow = await importAgentFlow(file);
    if (!sflow.nodes || !sflow.edges) return;
    const flow = deserializeAgentFlow(sflow, agent_defs);
    nodes = flow.nodes;
    edges = flow.edges;
    flowName = flow.name;
  }

  async function onAddAgent(agent_name: string) {
    const snode = newAgentFlowNode(agent_name, agent_defs);
    const xy = screenToFlowPosition({
      x: window.innerWidth * 0.45,
      y: window.innerHeight * 0.3,
    });
    snode.x = xy.x;
    snode.y = xy.y;
    await addAgentFlowNode(flowName, snode);
    const new_node = deserializeAgentFlowNode(snode, agent_defs);
    flows[flowName].nodes.push(new_node);
    nodes = [...nodes, new_node];
  }

  async function onPlay() {
    for (const node of nodes) {
      if (!node.data.enabled) {
        updateNodeData(node.id, { enabled: true });
        await startAgent(node.id);
      }
    }
  }

  async function onPause() {
    for (const node of nodes) {
      if (node.data.enabled) {
        updateNodeData(node.id, { enabled: false });
        await stopAgent(node.id);
      }
    }
  }
</script>

<main class="container static min-w-[100vw]">
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    deleteKey={["Delete", "Backspace"]}
    connectionRadius={38}
    colorMode="dark"
    fitView
    maxZoom={2}
    minZoom={0.2}
    attributionPosition="bottom-left"
    class="relative w-full min-h-screen !text-black !dark:text-white !bg-gray-100 dark:!bg-black"
  >
    <Controls />
    <MiniMap />
    <ButtonGroup class="absolute bottom-4 z-10 w-full flex justify-center">
      <Button onclick={onPause} pill class="!bg-gray-800">
        <PauseOutline
          class="w-5 h-5 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500"
        />
      </Button>
      <Button onclick={onPlay} pill class="!bg-gray-800">
        <PlayOutline
          class="w-5 h-5 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500"
        />
      </Button>
    </ButtonGroup>
  </SvelteFlow>

  <Navbar class="fixed top-4 left-0 z-10 !bg-transparent">
    <NavUl>
      <NavLi>
        File<ChevronDownOutline class="w-6 h-6 ms-2 inline" />
      </NavLi>
      <Dropdown class="!bg-gray-100 dark:!bg-gray-900">
        <DropdownItem onclick={onNewFlow}>New</DropdownItem>
        <DropdownItem onclick={onSaveFlow}>Save</DropdownItem>
        <DropdownItem onclick={onExportFlow}>Export</DropdownItem>
        <DropdownItem onclick={onImportFlow}>Import</DropdownItem>
      </Dropdown>
      <NavLi>
        {flowName}<ChevronDownOutline class="w-6 h-6 ms-2 inline" />
      </NavLi>
      <FlowMegaMenu {flows} bind:flowName bind:open={openFlow} />
      <NavLi>
        Agents<ChevronDownOutline class="w-6 h-6 ms-2 inline" />
      </NavLi>
      <AgentMegaMenu agentDefs={data.agent_defs} {onAddAgent} bind:open={openAgent} />
    </NavUl>
  </Navbar>

  {#if newFlowModal}
    <Modal title="New Flow" bind:open={newFlowModal}>
      <div class="flex flex-col">
        <label for="flow_name" class="mb-2 text-sm font-medium text-gray-900 dark:text-white"
          >Flow Name</label
        >
        <input
          type="text"
          id="flow_name"
          bind:value={newFlowName}
          class="block p-2 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
          placeholder="Flow Name"
        />
      </div>
      <div class="flex justify-end mt-4">
        <GradientButton color="pinkToOrange" onclick={createNewFlow}>Create</GradientButton>
      </div>
    </Modal>
  {/if}
</main>

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 3;
    stroke-opacity: 0.75;
  }
</style>
