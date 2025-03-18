<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  const DEFAULT_HANDLE_STYLE = "width: 10px; height: 10px;";

  type Props = NodeProps & {
    inputs: string[];
    outputs: string[];
    title: Snippet;
    contents: Snippet;
  };

  let { id, inputs, outputs, title, contents }: Props = $props();

  const { deleteElements, getNode } = useSvelteFlow();

  async function deleteNode(event: MouseEvent) {
    event.preventDefault();
    const node = getNode(id);
    if (node !== undefined) {
      await deleteElements({ nodes: [node] });
    }
  }
</script>

<div class="relative">
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
    class="relative p-0 bg-white dark:bg-gray-800 text-black dark:text-white border-2 border-gray-700 rounded-xl shadow-xl"
  >
    <div class="flex justify-between items-center pl-4 pr-0 mb-2">
      {@render title()}
      <Button onclick={deleteNode}><CloseOutline /></Button>
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
</div>
