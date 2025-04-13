<script lang="ts" module>
  const bgColors = [
    "bg-zinc-100 dark:bg-zinc-900 opacity-70",
    "bg-slate-200 dark:bg-slate-800 opacity-90",
  ];

  const DEFAULT_HANDLE_STYLE = "width: 11px; height: 11px; background: white;";
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, NodeResizer, Position } from "@xyflow/svelte";
  import type { NodeProps, ResizeDragEvent, ResizeParams } from "@xyflow/svelte";

  import type { SAgentDefinition } from "@/lib/types";

  type Props = NodeProps & {
    data: {
      enabled: boolean;
    };
    agentDef: SAgentDefinition;
    title: Snippet;
    contents: Snippet;
  };

  let { data, agentDef, selected, height, title, contents }: Props = $props();

  const inputs = agentDef?.inputs ?? [];
  const outputs = agentDef?.outputs ?? [];

  let ht = $state(height);

  function onResize(_ev: ResizeDragEvent, params: ResizeParams) {
    ht = params.height;
  }
</script>

<NodeResizer isVisible={selected} {onResize} />
<div
  class="{bgColors[
    data.enabled ? 1 : 0
  ]} flex flex-col p-0 text-black dark:text-white border-2 border-gray-700 rounded-xl shadow-xl"
  style:height={ht ? `${ht}px` : "auto"}
>
  <div class="w-full flex-none ml-4 mr-8 mb-2">
    {@render title()}
  </div>
  <div class="w-full flex-none grid grid-cols-2 gap-1 mb-4">
    <div>
      {#each inputs as input}
        <div class="text-left ml-2">
          {input}
        </div>
      {/each}
    </div>
    <div>
      {#each outputs as output}
        <div class="text-right mr-2">
          {output}
        </div>
      {/each}
    </div>
  </div>
  <div class="w-full grow flex flex-col gap-2">
    {@render contents()}
  </div>
</div>
{#each inputs as input, idx}
  <Handle
    id={input}
    type="target"
    position={Position.Left}
    style="top: {idx * 28 + 59}px; {DEFAULT_HANDLE_STYLE}"
  />
{/each}
{#each outputs as output, idx}
  <Handle
    id={output}
    type="source"
    position={Position.Right}
    style="top: {idx * 28 + 59}px; {DEFAULT_HANDLE_STYLE}"
  />
{/each}
