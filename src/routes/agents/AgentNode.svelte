<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import { useNodes } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Card, Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  import { getAgentSettingsContext } from "@/lib/agent";
  import type { AgentConfig } from "@/lib/types";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: Writable<boolean>;
      config: Record<string, AgentConfig>;
    };
  };

  let { id, data }: Props = $props();

  const nodes = useNodes();
  const agent_schema = getAgentSettingsContext()?.[data.name]?.schema;

  function deleteNode() {
    nodes.update((nodes) => {
      nodes = nodes.filter((node) => node.id !== id);
      return nodes;
    });
  }
</script>

<div>
  <Card padding="none">
    <div class="flex justify-between items-center pl-4 pr-0 mb-2">
      <h3 class="text-xl pt-2">{agent_schema?.["title"] || data.name}</h3>
      <Button onclick={deleteNode}><CloseOutline /></Button>
    </div>
    <h4 class="text-sm pl-4 pb-4">{agent_schema?.["description"] || ""}</h4>
    <form class="grid grid-cols-6 gap-4 p-4">
      <Toggle bind:checked={() => get(data.enabled), (v) => data.enabled.set(v)} class="col-span-6"
      ></Toggle>
      {#each Object.keys(data.config) as key}
        {@const config = data.config[key]}
        <Label class="col-span-6 space-y-2">
          <h3>{config.title || key}</h3>
          <p class="text-xs text-gray-500">{config["description"]}</p>
          {#if config.type === "boolean"}
            <Toggle bind:checked={() => get(config.value), (v) => config.value.set(v)} />
          {:else if config.type === "integer"}
            <NumberInput bind:value={() => get(config.value), (v) => config.value.set(v)} />
          {:else if config.type === "number"}
            <Input type="text" bind:value={() => get(config.value), (v) => config.value.set(v)} />
          {:else if config.type === "string" || config.type === "string?"}
            <Input type="text" bind:value={() => get(config.value), (v) => config.value.set(v)} />
          {:else if config.type === "string[]"}
            <Textarea bind:value={() => get(config.value), (v) => config.value.set(v)} rows={4} />
          {:else}
            <Input type="text" value={get(config.value)} disabled />
          {/if}
        </Label>
      {/each}
    </form>
  </Card>
</div>
