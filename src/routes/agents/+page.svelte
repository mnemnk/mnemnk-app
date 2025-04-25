<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import { getContext, onMount } from "svelte";

  import {
    useSvelteFlow,
    Background,
    BackgroundVariant,
    Controls,
    MiniMap,
    SvelteFlow,
    type Edge,
    type Node,
    type NodeTypes,
  } from "@xyflow/svelte";
  // 👇 this is important! You need to import the styles for Svelte Flow to work
  import "@xyflow/svelte/dist/style.css";
  import { Button, ButtonGroup, GradientButton, Modal } from "flowbite-svelte";
  import { ExclamationCircleOutline, PauseOutline, PlayOutline } from "flowbite-svelte-icons";
  import hotkeys from "hotkeys-js";

  import {
    addAgentFlowEdge,
    addAgentFlowNode,
    removeAgentFlowEdge,
    removeAgentFlowNode,
    deserializeAgentFlow,
    deserializeAgentFlowEdge,
    deserializeAgentFlowNode,
    importAgentFlow,
    newAgentFlow,
    newAgentFlowNode,
    saveAgentFlow,
    serializeAgentFlow,
    serializeAgentFlowEdge,
    serializeAgentFlowNode,
    setAgentDefinitionsContext,
    startAgent,
    stopAgent,
    renameAgentFlow,
    deleteAgentFlow,
    copySubFlow,
    insertAgentFlow,
  } from "@/lib/agent";
  import { flowNameState } from "@/lib/shared.svelte";
  import type {
    AgentFlowNode,
    AgentFlowEdge,
    SAgentFlowNode,
    SAgentFlowEdge,
    AgentFlow,
  } from "@/lib/types";

  import AgentList from "./AgentList.svelte";
  import AgentNode from "./AgentNode.svelte";
  import FileMenu from "./FileMenu.svelte";
  import FlowList from "./FlowList.svelte";
  import NodeContextMenu from "./NodeContextMenu.svelte";

  let { data } = $props();

  const { getViewport, screenToFlowPosition, setViewport, updateEdge, updateNode, updateNodeData } =
    $derived(useSvelteFlow());
  setAgentDefinitionsContext(data.agentDefs);

  const nodeTypes: NodeTypes = {
    agent: AgentNode,
  };

  let nodes = $state.raw<AgentFlowNode[]>([]);
  let edges = $state.raw<AgentFlowEdge[]>([]);

  const agentDefs = data.agentDefs;
  const flows = getContext<() => Record<string, AgentFlow>>("agentFlows");

  let flowNames = $state.raw<string[]>([]);

  let flowActivities = $state<Record<string, boolean>>({});

  function updateNodesAndEdges() {
    nodes = [...flows()[flowNameState.name].nodes];
    edges = [...flows()[flowNameState.name].edges];
    const viewport = flows()[flowNameState.name].viewport;
    if (viewport) {
      setViewport(viewport);
    }
  }

  function updateFlowNames() {
    flowNames = Object.keys(flows()).sort();
  }

  function updateFlowActivities() {
    flowActivities = Object.fromEntries(
      Object.entries(flows()).map(([name, flow]) => [
        name,
        flow.nodes.some((node) => node.data.enabled),
      ]),
    );
  }

  function updateCurrentFlowActivity() {
    flowActivities[flowNameState.name] = nodes.some((node) => node.data.enabled);
  }

  onMount(() => {
    updateNodesAndEdges();
    updateFlowNames();
    updateFlowActivities();
    return async () => {
      await syncFlow();
    };
  });

  async function changeFlowName(name: string) {
    await syncFlow();
    flowNameState.name = name;
    updateNodesAndEdges();
  }

  async function handleOnDelete(params: { nodes: Node[]; edges: Edge[] }) {
    if (params.edges && params.edges.length > 0) {
      await checkEdgeChange(edges);
    }
    if (params.nodes && params.nodes.length > 0) {
      await checkNodeChange(nodes);
      updateCurrentFlowActivity();
    }
  }

  async function handleOnConnect() {
    await checkEdgeChange(edges);
  }

  async function checkNodeChange(nodes: AgentFlowNode[]) {
    const nodeIds = new Set(nodes.map((node) => node.id));

    const deletedNodes = flows()[flowNameState.name]?.nodes.filter((node) => !nodeIds.has(node.id));
    if (deletedNodes) {
      for (const node of deletedNodes) {
        await removeAgentFlowNode(flowNameState.name, node.id);
        flows()[flowNameState.name].nodes = flows()[flowNameState.name].nodes.filter(
          (n) => n.id !== node.id,
        );
      }
    }
  }

  async function checkEdgeChange(edges: AgentFlowEdge[]) {
    const edgeIds = new Set(edges.map((edge) => edge.id));

    const deletedEdges = flows()[flowNameState.name]?.edges.filter((edge) => !edgeIds.has(edge.id));
    if (deletedEdges) {
      for (const edge of deletedEdges) {
        await removeAgentFlowEdge(flowNameState.name, edge.id);
        flows()[flowNameState.name].edges = flows()[flowNameState.name].edges.filter(
          (e) => e.id !== edge.id,
        );
      }
    }

    const addedEdges = edges.filter(
      (edge) => !flows()[flowNameState.name].edges.some((e) => e.id === edge.id),
    );
    for (const edge of addedEdges) {
      await addAgentFlowEdge(flowNameState.name, serializeAgentFlowEdge(edge));
      flows()[flowNameState.name].edges.push(edge);
    }
  }

  async function syncFlow() {
    const viewport = getViewport();
    const flow = serializeAgentFlow(nodes, edges, flowNameState.name, agentDefs, viewport);
    flows()[flowNameState.name] = deserializeAgentFlow(flow, agentDefs);
    await insertAgentFlow(flow);
  }

  // cut, copy and paste

  let copiedNodes = $state.raw<SAgentFlowNode[]>([]);
  let copiedEdges = $state.raw<SAgentFlowEdge[]>([]);

  function selectedNodesAndEdges(): [AgentFlowNode[], AgentFlowEdge[]] {
    const selectedNodes = nodes.filter((node) => node.selected);
    const selectedEdges = edges.filter((edge) => edge.selected);
    return [selectedNodes, selectedEdges];
  }

  async function cutNodesAndEdges() {
    const [selectedNodes, selectedEdges] = selectedNodesAndEdges();
    if (selectedNodes.length == 0 && selectedEdges.length == 0) {
      return;
    }
    copiedNodes = selectedNodes.map((node) => serializeAgentFlowNode(node, agentDefs));
    copiedEdges = selectedEdges.map((edge) => serializeAgentFlowEdge(edge));

    nodes = nodes.filter((node) => !node.selected);
    edges = edges.filter((edge) => !edge.selected);
    await checkNodeChange(nodes);
    await checkEdgeChange(edges);
    updateCurrentFlowActivity();
  }

  function copyNodesAndEdges() {
    const [selectedNodes, selectedEdges] = selectedNodesAndEdges();
    if (selectedNodes.length == 0) {
      return;
    }
    copiedNodes = selectedNodes.map((node) => serializeAgentFlowNode(node, agentDefs));
    copiedEdges = selectedEdges.map((edge) => serializeAgentFlowEdge(edge));
  }

  async function pasteNodesAndEdges() {
    nodes.forEach((node) => {
      if (node.selected) {
        updateNode(node.id, { selected: false });
      }
    });
    edges.forEach((edge) => {
      if (edge.selected) {
        updateEdge(edge.id, { selected: false });
      }
    });

    if (copiedNodes.length == 0) {
      return;
    }

    let [cnodes, cedges] = await copySubFlow(copiedNodes, copiedEdges);
    if (cnodes.length == 0 && cedges.length == 0) return;

    let new_nodes = [];
    for (const node of cnodes) {
      node.x += 80;
      node.y += 80;
      node.enabled = false;
      await addAgentFlowNode(flowNameState.name, node);
      const new_node = deserializeAgentFlowNode(node, agentDefs);
      new_node.selected = true;
      new_nodes.push(new_node);
      flows()[flowNameState.name].nodes.push(new_node);
    }

    let new_edges = [];
    for (const edge of cedges) {
      await addAgentFlowEdge(flowNameState.name, edge);
      const new_edge = deserializeAgentFlowEdge(edge);
      new_edge.selected = true;
      new_edges.push(new_edge);
      flows()[flowNameState.name].edges.push(new_edge);
    }

    nodes = [...nodes, ...new_nodes];
    edges = [...edges, ...new_edges];
  }

  function selectAllNodesAndEdges() {
    nodes.forEach((node) => {
      updateNode(node.id, { selected: true });
    });
    edges.forEach((edge) => {
      updateEdge(edge.id, { selected: true });
    });
  }

  // shortcuts

  let hiddenAgents = $state(true);
  const key_open_agent = "a";

  let openFlow = $state(false);
  const key_open_flow = "f";

  $effect(() => {
    hotkeys("ctrl+r", (event) => {
      event.preventDefault();
    });

    hotkeys("ctrl+s", (event) => {
      event.preventDefault();
      onSaveFlow();
    });

    hotkeys(key_open_agent, () => {
      hiddenAgents = !hiddenAgents;
    });

    hotkeys(key_open_flow, () => {
      openFlow = !openFlow;
    });

    hotkeys("ctrl+x", () => {
      /* await */ cutNodesAndEdges();
    });
    hotkeys("ctrl+c", () => {
      copyNodesAndEdges();
    });
    hotkeys("ctrl+v", () => {
      pasteNodesAndEdges();
    });
    hotkeys("ctrl+a", (ev) => {
      ev.preventDefault();
      selectAllNodesAndEdges();
    });

    return () => {
      hotkeys.unbind("ctrl+r");
      hotkeys.unbind("ctrl+s");
      hotkeys.unbind(key_open_agent);
      hotkeys.unbind(key_open_flow);
      hotkeys.unbind("ctrl+x");
      hotkeys.unbind("ctrl+c");
      hotkeys.unbind("ctrl+v");
      hotkeys.unbind("ctrl+a");
    };
  });

  // New Flow

  let newFlowModal = $state(false);
  let newFlowName = $state("");

  function onNewFlow() {
    newFlowName = "";
    newFlowModal = true;
  }

  async function createNewFlow() {
    newFlowModal = false;
    if (!newFlowName) return;
    const flow = await newAgentFlow(newFlowName);
    if (!flow) return;
    flows()[flow.name] = deserializeAgentFlow(flow, agentDefs);
    updateFlowNames();
    updateFlowActivities();
    await changeFlowName(flow.name);
  }

  // Rename Flow

  let renameFlowModal = $state(false);
  let renameFlowName = $state("");

  function onRenameFlow() {
    renameFlowName = flowNameState.name;
    renameFlowModal = true;
  }

  async function renameFlow() {
    renameFlowModal = false;
    if (!renameFlowName || renameFlowName === flowNameState.name) return;
    const oldName = flowNameState.name;
    const newName = await renameAgentFlow(flowNameState.name, renameFlowName);
    if (!newName) return;
    const flow = flows()[oldName];
    flow.name = newName;
    flows()[newName] = flow;
    delete flows()[oldName];
    updateFlowNames();
    updateFlowActivities();
    // We don't need to sync the current flow.
    // await changeFlowName(newName);
    flowNameState.name = newName;
    updateNodesAndEdges();
  }

  // Delete Flow

  let deleteFlowModal = $state(false);

  function onDeleteFlow() {
    deleteFlowModal = true;
  }

  async function deleteFlow() {
    deleteFlowModal = false;
    const flow = flows()[flowNameState.name];
    if (!flow) return;
    await deleteAgentFlow(flowNameState.name);
    delete flows()[flowNameState.name];
    updateFlowNames();
    updateFlowActivities();
    // TODO: create a new flow when deleting the main flow
    flowNameState.name = "main";
    updateNodesAndEdges();
  }

  async function onSaveFlow() {
    if (flowNameState.name in flows()) {
      const viewport = getViewport();
      const flow = serializeAgentFlow(nodes, edges, flowNameState.name, agentDefs, viewport);
      await saveAgentFlow(flow);
      flows()[flowNameState.name] = deserializeAgentFlow(flow, agentDefs);
    }
  }

  function onExportFlow() {
    const viewport = getViewport();
    const flow = serializeAgentFlow(nodes, edges, flowNameState.name, agentDefs, viewport);
    const jsonStr = JSON.stringify(flow, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = flowNameState.name + ".json";
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
    const flow = deserializeAgentFlow(sflow, agentDefs);
    flows()[flow.name] = flow;
    updateFlowNames();
    updateFlowActivities();
    await changeFlowName(flow.name);
  }

  async function onAddAgent(agent_name: string) {
    const snode = newAgentFlowNode(agent_name, agentDefs);
    const xy = screenToFlowPosition({
      x: window.innerWidth * 0.45,
      y: window.innerHeight * 0.3,
    });
    snode.x = xy.x;
    snode.y = xy.y;
    await addAgentFlowNode(flowNameState.name, snode);
    const new_node = deserializeAgentFlowNode(snode, agentDefs);
    flows()[flowNameState.name].nodes.push(new_node);
    nodes = [...nodes, new_node];
  }

  async function onPlay() {
    const [selectedNodes, selectedEdges] = selectedNodesAndEdges();
    if (selectedNodes.length > 0 || selectedEdges.length > 0) {
      // start only selected agents
      for (const node of selectedNodes) {
        if (!node.data.enabled) {
          updateNodeData(node.id, { enabled: true });
          await startAgent(node.id);
        }
      }
      updateCurrentFlowActivity();
      return;
    }

    // start all agents
    for (const node of nodes) {
      if (!node.data.enabled) {
        updateNodeData(node.id, { enabled: true });
        await startAgent(node.id);
      }
    }
    updateCurrentFlowActivity();
  }

  async function onPause() {
    const [selectedNodes, selectedEdges] = selectedNodesAndEdges();
    if (selectedNodes.length > 0 || selectedEdges.length > 0) {
      // stop only selected agents
      for (const node of selectedNodes) {
        if (node.data.enabled) {
          updateNodeData(node.id, { enabled: false });
          await stopAgent(node.id);
        }
      }
      updateCurrentFlowActivity();
      return;
    }

    // stop all agents
    for (const node of nodes) {
      if (node.data.enabled) {
        updateNodeData(node.id, { enabled: false });
        await stopAgent(node.id);
      }
    }
    updateCurrentFlowActivity();
  }

  let nodeContextMenu: {
    x: number;
    y: number;
  } | null = $state(null);

  function showNodeContextMenu(x: number, y: number) {
    nodeContextMenu = {
      x,
      y,
    };
  }

  function hideNodeContextMenu() {
    nodeContextMenu = null;
  }

  function handleNodeContextMenu({ event, node }: { event: MouseEvent; node: Node }) {
    event.preventDefault();

    const agentNode = node as unknown as AgentFlowNode;

    const [selectedNodes, _] = selectedNodesAndEdges();
    if (!selectedNodes.some((n) => n.id === agentNode.id)) {
      nodes.forEach((n) => {
        updateNode(n.id, { selected: false });
      });
      updateNode(agentNode.id, { selected: true });
    }

    showNodeContextMenu(event.clientX, event.clientY);
  }

  function handleSelectionContextMenu({ event }: { event: MouseEvent }) {
    event.preventDefault();
    showNodeContextMenu(event.clientX, event.clientY);
  }

  function handleNodeClick() {
    hideNodeContextMenu();
  }

  function handleSelectionClick() {
    hideNodeContextMenu();
  }

  function handlePaneClick() {
    hideNodeContextMenu();
  }
</script>

<div class="bg-white! dark:bg-black! static">
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    onnodecontextmenu={handleNodeContextMenu}
    onselectioncontextmenu={handleSelectionContextMenu}
    onnodeclick={handleNodeClick}
    onselectionclick={handleSelectionClick}
    onpaneclick={handlePaneClick}
    ondelete={handleOnDelete}
    onconnect={handleOnConnect}
    deleteKey={["Delete"]}
    connectionRadius={38}
    colorMode="dark"
    fitView
    maxZoom={2}
    minZoom={0.1}
    attributionPosition="bottom-left"
    class="relative w-full min-h-screen text-black! !dark:text-white bg-gray-100! dark:bg-black!"
  >
    <Background
      variant={BackgroundVariant.Lines}
      bgColor="#000"
      patternColor="#1a1a1a"
      gap={28}
      lineWidth={1}
    />
    <Controls />
    <MiniMap />
    <ButtonGroup class="absolute bottom-4 z-10 w-full flex justify-center">
      <Button onclick={onPause} pill class="bg-gray-100! dark:bg-gray-900! opacity-80">
        <PauseOutline
          class="w-5 h-5 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500"
        />
      </Button>
      <Button onclick={onPlay} pill class="bg-gray-100! dark:bg-gray-900! opacity-80">
        <PlayOutline
          class="w-5 h-5 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500"
        />
      </Button>
    </ButtonGroup>

    {#if nodeContextMenu}
      <NodeContextMenu
        x={nodeContextMenu.x}
        y={nodeContextMenu.y}
        {hideNodeContextMenu}
        onstart={onPlay}
        onstop={onPause}
        oncut={cutNodesAndEdges}
        oncopy={copyNodesAndEdges}
      />
    {/if}

    <FileMenu
      {onNewFlow}
      {onRenameFlow}
      {onDeleteFlow}
      {onSaveFlow}
      {onExportFlow}
      {onImportFlow}
    />
  </SvelteFlow>
  <div class="absolute top-1 left-0 w-40">
    <FlowList {flowNames} currentFlowName={flowNameState.name} {flowActivities} {changeFlowName} />
  </div>
  <div class="absolute right-0 top-1 w-60">
    <AgentList {agentDefs} {onAddAgent} />
  </div>
</div>

{#if newFlowModal}
  <Modal title="New Flow" bind:open={newFlowModal} classBackdrop="bg-transparent backdrop-blur-xs">
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
        onkeydown={(e) => {
          if (e.key === "Enter") {
            createNewFlow();
          }
        }}
      />
    </div>
    <div class="flex justify-end mt-4">
      <GradientButton color="pinkToOrange" onclick={createNewFlow}>Create</GradientButton>
    </div>
  </Modal>
{/if}

{#if renameFlowModal}
  <Modal
    title="Rename Flow"
    bind:open={renameFlowModal}
    classBackdrop="bg-transparent backdrop-blur-xs"
  >
    <div class="flex flex-col">
      <label for="flow_name" class="mb-2 text-sm font-medium text-gray-900 dark:text-white"
        >Flow Name</label
      >
      <input
        type="text"
        id="flow_name"
        bind:value={renameFlowName}
        class="block p-2 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
        placeholder="Flow Name"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            renameFlow();
          }
        }}
      />
    </div>
    <div class="flex justify-end mt-4">
      <GradientButton color="pinkToOrange" onclick={renameFlow}>Rename</GradientButton>
    </div>
  </Modal>
{/if}

{#if deleteFlowModal}
  <Modal
    title="Delete Flow"
    bind:open={deleteFlowModal}
    size="xs"
    autoclose
    classBackdrop="bg-transparent backdrop-blur-xs"
  >
    <div class="text-center">
      <ExclamationCircleOutline class="mx-auto mb-4 text-gray-400 w-12 h-12 dark:text-gray-200" />
      <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">
        Are you sure you want to delete this flow?
      </h3>
      <Button onclick={deleteFlow} color="red" class="me-2">Delete</Button>
      <Button color="alternative">Cancel</Button>
    </div>
  </Modal>
{/if}

<style>
  :global(.svelte-flow__edge .svelte-flow__edge-path) {
    stroke-width: 6px;
    stroke-opacity: 0.8;
  }
</style>
