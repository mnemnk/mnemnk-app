<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import { Handle, Position, useNodes } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Card, Input, Label, Toggle } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  import type { AgentConfig } from "@/lib/types";

  const board_name_key = "board_name";
  const persistent_key = "persistent";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: Writable<boolean>;
      config: AgentConfig;
    };
  };

  let { id, data }: Props = $props();

  const nodes = useNodes();

  function deleteNode() {
    nodes.update((nodes) => {
      nodes = nodes.filter((node) => node.id !== id);
      return nodes;
    });
  }
</script>

<div>
  <Handle type="target" position={Position.Left} />
  <Card padding="none">
    <div class="flex justify-between items-center pl-4 pr-0 mb-2">
      <h3 class="text-xl pt-2">{data.name}</h3>
      <Button onclick={deleteNode}><CloseOutline /></Button>
    </div>
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
      <Label class="col-span-6 space-y-2">
        <h3>Persistent</h3>
        <Toggle
          bind:checked={
            () => get(data.config[persistent_key].value),
            (v) => data.config[persistent_key].value.set(v)
          }
        />
      </Label>
    </form>
  </Card>
  <Handle type="source" position={Position.Right} />
</div>
