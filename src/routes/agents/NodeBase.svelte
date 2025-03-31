<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Toggle } from "flowbite-svelte";

  const DEFAULT_HANDLE_STYLE = "width: 10px; height: 10px;";

  type Props = NodeProps & {
    enabled: boolean;
    inputs: string[];
    outputs: string[];
    title: Snippet;
    contents: Snippet;
  };

  const bgColors = ["bg-gray-200 dark:bg-gray-700", "bg-gray-100 dark:bg-gray-800"];

  let { id, enabled, inputs, outputs, title, contents }: Props = $props();
  const { updateNodeData } = useSvelteFlow();
</script>

{#each inputs as input, idx}
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
  class={`${bgColors[enabled ? 1 : 0]} p-0 text-black dark:text-white border-2 border-gray-700 rounded-xl shadow-xl`}
>
  <div class="w-full flex justify-between pl-4 pr-0 mb-2">
    {@render title()}
    <div class="flex-none w-8"></div>
    <Toggle
      checked={enabled}
      onchange={() => updateNodeData(id, { enabled: !enabled })}
      size="custom"
      customSize="w-8 h-4 after:top-0 after:left-[2px]  after:h-4 after:w-4"
      class="col-span-6 pt-1"
    ></Toggle>
  </div>
  {@render contents()}
</div>
{#each outputs as output, idx}
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
