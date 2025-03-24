<script lang="ts">
  import { quadOut } from "svelte/easing";

  import type { NodeProps } from "@xyflow/svelte";

  import { MESSAGES_TIMEOUT } from "@/lib/shared.svelte";

  // extends Props from NodeProps
  type Props = NodeProps & {
    data: {
      value: Record<string, any>;
      time: number;
    };
  };

  const BACKGROUND = [
    "bg-gray-200",
    "bg-gray-200/90",
    "bg-gray-200/80",
    "bg-gray-200/70",
    "bg-gray-200/60",
    "bg-gray-200/50",
    "bg-gray-200/40",
    "bg-gray-200/30",
    "bg-gray-200/20",
    "bg-gray-200/10",
  ];

  let { data }: Props = $props();

  let opacity = $state(1.0);

  $effect(() => {
    const interval = setInterval(() => {
      opacity = 1.0 - quadOut((Date.now() - data.time) / MESSAGES_TIMEOUT);
    }, 1000);

    return () => {
      clearInterval(interval);
    };
  });
</script>

<div class="bg-gray-200 max-w-[400px] p-4 rounded-lg" style="opacity: {opacity}">
  <h3 class="text-lg">{new Date(data.time).toLocaleTimeString()}</h3>
  {#each Object.entries(data.value) as [key, value]}
    {#if key !== "t"}
      <div class="overflow-hidden text-wrap">
        <span>{key}</span>: <span>{value}</span>
      </div>
    {/if}
  {/each}
</div>
