<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Input, Label, Toggle } from "flowbite-svelte";

  import type { AgentConfigEntry } from "@/lib/types";

  import NodeBase from "./NodeBase.svelte";

  const board_name_key = "board_name";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: Writable<boolean>;
      inputs: string[];
      outputs: string[];
      config: {
        board_name: AgentConfigEntry;
      };
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
    <Label class="col-span-6 space-y-2">
      <h3>Board Name</h3>
      <Input
        type="text"
        bind:value={
          () => get(data.config[board_name_key].value),
          (v) => data.config[board_name_key].value.set(v)
        }
      />
    </Label>
  </form>
{/snippet}

<NodeBase {id} inputs={data.inputs} outputs={data.outputs} {title} {contents} />
