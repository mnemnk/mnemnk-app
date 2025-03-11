<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import { Card, Heading } from "flowbite-svelte";

  // extends Props from NodeProps
  type Props = NodeProps & {
    data: {
      agent: string;
      value: object;
    };
  };

  let { data }: Props = $props();
  const uid = $props.id();
</script>

<div>
  <Card class="opacity-90">
    <Heading tag="h4">
      {data.agent}
    </Heading>
    <div>
      {#each Object.entries(data.value) as [key, value]}
        <div>
          <span>{key}</span>: <span>{value}</span>
        </div>
      {/each}
    </div>
  </Card>
</div>
