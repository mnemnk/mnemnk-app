<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";

  // extends Props from NodeProps
  type Props = NodeProps & {
    data: {
      kind: string;
    };
  };

  const COLORS = [
    "bg-teal-500/50",
    "bg-sky-500/50",
    "bg-rose-500/50",
    "bg-purple-500/50",
    "bg-lime-500/50",
  ];

  let { id, data }: Props = $props();

  // hash number of kind to get a color
  const color =
    data.kind.split("").reduce((acc, char) => acc * 17 + char.charCodeAt(0), 3) % COLORS.length;
</script>

<div {id} class={`${COLORS[color]} w-full h-full p-4 rounded-lg`}>
  <h2 class="text-white text-2xl">{data.kind}</h2>
</div>
