<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, NodeResizer, Position, useSvelteFlow } from "@xyflow/svelte";
  import type { NodeProps, ResizeDragEvent, ResizeParams } from "@xyflow/svelte";
  import { Toggle } from "flowbite-svelte";

  import { startAgent, stopAgent } from "@/lib/agent";

  const DEFAULT_HANDLE_STYLE = "width: 10px; height: 10px;";

  type Props = NodeProps & {
    data: {
      enabled: boolean;
      inputs: string[];
      outputs: string[];
    };
    title: Snippet;
    contents: Snippet;
  };

  let { id, data, selected, height, title, contents }: Props = $props();

  const bgColors = ["bg-zinc-100 dark:bg-zinc-900", "bg-slate-100 dark:bg-slate-900"];

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
{#each data.inputs as input, idx}
  <Handle
    id={input}
    type="target"
    position={Position.Left}
    style={`top: ${idx * 30 + 50}px; ${DEFAULT_HANDLE_STYLE}`}
  />
  <div
    class="absolute text-white opacity-20 hover:opacity-100"
    style={`top: ${idx * 30 + 30}px; left: -20px;`}
  >
    {input}
  </div>
{/each}
<div
  class={`${bgColors[data.enabled ? 1 : 0]} p-0 text-black dark:text-white border-2 border-gray-700 rounded-xl shadow-xl`}
  style:height={ht ? `${ht}px` : "auto"}
>
  <div class="w-full flex justify-between pl-4 pr-0 mb-2">
    {@render title()}
    <div class="flex-none w-8"></div>
    <Toggle
      checked={data.enabled}
      onchange={() => updateEnabled(!data.enabled)}
      size="custom"
      customSize="w-8 h-4 after:top-0 after:left-[2px]  after:h-4 after:w-4"
      class="col-span-6 pt-1"
    ></Toggle>
  </div>
  {@render contents()}
</div>
{#each data.outputs as output, idx}
  <div
    class="absolute text-white opacity-20 hover:opacity-100"
    style={`top: ${idx * 30 + 30}px; left: 105%;`}
  >
    {output}
  </div>
  <Handle
    id={output}
    type="source"
    position={Position.Right}
    style={`top: ${idx * 30 + 50}px; ${DEFAULT_HANDLE_STYLE}`}
  />
{/each}
