<script lang="ts" module>
  const bgColors = ["bg-zinc-100 dark:bg-zinc-900", "bg-slate-100 dark:bg-slate-900"];

  const DEFAULT_HANDLE_STYLE =
    "width: 11px; height: 11px; border-width: 2px; background: black; border-color: white;";
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, NodeResizer, Position, useSvelteFlow } from "@xyflow/svelte";
  import type { NodeProps, ResizeDragEvent, ResizeParams } from "@xyflow/svelte";
  import { Toggle } from "flowbite-svelte";

  import { startAgent, stopAgent } from "@/lib/agent";
  import type { SAgentDefinition } from "@/lib/types";

  type Props = NodeProps & {
    data: {
      enabled: boolean;
    };
    agentDef: SAgentDefinition;
    title: Snippet;
    contents: Snippet;
  };

  let { id, data, agentDef, selected, height, title, contents }: Props = $props();

  const inputs = agentDef?.inputs ?? [];
  const outputs = agentDef?.outputs ?? [];

  const { updateNodeData } = useSvelteFlow();

  async function updateEnabled(value: boolean) {
    updateNodeData(id, { enabled: value });
    if (value) {
      await startAgent(id);
    } else {
      await stopAgent(id);
    }
  }

  let ht = $state(height);

  function onResize(_ev: ResizeDragEvent, params: ResizeParams) {
    ht = params.height;
  }
</script>

<NodeResizer isVisible={selected} {onResize} />
{#each inputs as input, idx}
  <Handle
    id={input}
    type="target"
    position={Position.Left}
    style={`top: ${idx * 28 + 59}px; ${DEFAULT_HANDLE_STYLE}`}
  />
{/each}
<div
  class={`${bgColors[data.enabled ? 1 : 0]} flex flex-col p-0 text-black dark:text-white border-2 border-gray-700 rounded-xl shadow-xl`}
  style:height={ht ? `${ht}px` : "auto"}
>
  <div class="w-full flex-none flex flex-row items-center justify-between pl-4 pb-2">
    {@render title()}
    <div class="grow w-8"></div>
    <Toggle
      checked={data.enabled}
      onchange={() => updateEnabled(!data.enabled)}
      size="custom"
      customSize="w-8 h-4 after:top-0 after:left-[2px]  after:h-4 after:w-4"
      class="flex-none pt-1"
      tabindex={-1}
    ></Toggle>
  </div>
  <div class="w-full grow flex flex-col">
    <div class="w-full flex-none mb-2">
      {#each inputs as input}
        <div class="w-full text-left pl-2 mb-1">
          {input}
        </div>
      {/each}
      {#each outputs as output}
        <div class="w-full text-right pr-2 mb-1">
          {output}
        </div>
      {/each}
    </div>
    {@render contents()}
  </div>
</div>
{#each outputs as output, idx}
  <Handle
    id={output}
    type="source"
    position={Position.Right}
    style={`top: ${(inputs.length + idx) * 28 + 59}px; ${DEFAULT_HANDLE_STYLE}`}
  />
{/each}
