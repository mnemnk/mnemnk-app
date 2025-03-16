<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Toggle } from "flowbite-svelte";

  import NodeBase from "./NodeBase.svelte";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: Writable<boolean>;
      config: {};
    };
  };

  let { id, data }: Props = $props();
</script>

{#snippet title()}
  <h3 class="text-xl pt-2">{data.name}</h3>
{/snippet}

{#snippet contents()}
  <form class="grid grid-cols-6 gap-4 p-4">
    <Toggle bind:checked={() => get(data.enabled), (v) => data.enabled.set(v)} class="col-span-6"
    ></Toggle>
  </form>
{/snippet}

<NodeBase {id} {title} {contents} />
