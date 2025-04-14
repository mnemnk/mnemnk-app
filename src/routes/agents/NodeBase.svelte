<script lang="ts" module>
  const bgColors = [
    "bg-zinc-100 dark:bg-zinc-900 opacity-50 border-[#555]",
    "bg-slate-200 dark:bg-[#353535] opacity-90 border-[#777]",
  ];

  const DEFAULT_HANDLE_STYLE = "width: 11px; height: 11px;";

  const HANDLE_OFFSET = 71;
  const HANDLE_GAP = 25.5;
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
    titleColor: string;
    title: Snippet;
    contents: Snippet;
  };

  let { data, agentDef, selected, height, titleColor, title, contents }: Props = $props();

  const inputs = agentDef?.inputs ?? [];
  const outputs = agentDef?.outputs ?? [];

  let ht = $state(height);

  function onResize(_ev: ResizeDragEvent, params: ResizeParams) {
    ht = params.height;
  }
</script>

<NodeResizer isVisible={selected} variant="line" {onResize} />
<div
  class="{bgColors[
    data.enabled ? 1 : 0
  ]} flex flex-col p-0 text-black dark:text-white border-2 rounded-xl"
  style:height={ht ? `${ht}px` : "auto"}
>
  <div class="w-full min-w-40 flex-none pl-4 pr-4 pb-2 {titleColor} rounded-t-lg">
    {@render title()}
  </div>
  <div class="w-full flex-none grid grid-cols-2 gap-1 mt-4 mb-2">
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
    style="top: {idx * HANDLE_GAP + HANDLE_OFFSET}px; {DEFAULT_HANDLE_STYLE}"
  />
{/each}
{#each outputs as output, idx}
  <Handle
    id={output}
    type="source"
    position={Position.Right}
    style="top: {idx * HANDLE_GAP + HANDLE_OFFSET}px; {DEFAULT_HANDLE_STYLE}"
  />
{/each}
