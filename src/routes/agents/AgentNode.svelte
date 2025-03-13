<script lang="ts">
  import { useNodes } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Card, Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  import { save_agent_config, set_agent_enabled, start_agent, stop_agent } from "@/lib/utils";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: boolean;
      config: Record<string, any>;
      schema?: Record<string, any>;
      properties?: Map<string, any>;
    };
  };

  let { id, data }: Props = $props();

  const nodes = useNodes();

  async function toggle_agent() {
    if (data.enabled) {
      data.enabled = false;
      await set_agent_enabled(id, false);
      await stop_agent(id);
    } else {
      data.enabled = true;
      await start_agent(id);
      await set_agent_enabled(id, true);
    }
  }

  async function save() {
    if (data.properties) {
      const properties = data.properties;
      let config = { ...data.config };
      if (properties && config) {
        for (const [key, value] of properties.entries()) {
          if (value.type === "boolean") {
            config = {
              ...config,
              [key]: value.value === "true",
            };
          } else if (value.type === "integer") {
            config = {
              ...config,
              [key]: parseInt(value.value),
            };
          } else if (value.type === "number") {
            config = {
              ...config,
              [key]: parseFloat(value.value),
            };
          } else if (value.type === "string") {
            console.log(config);
            config = {
              ...config,
              [key]: value.value,
            };
            console.log(config);
          } else if (value.type === "string[]") {
            config = {
              ...config,
              [key]: value.value.split("\n"),
            };
          } else if (value.type === "string?") {
            config = {
              ...config,
              [key]: value.value === "" ? null : value.value,
            };
          }
        }
        data.config = config;
        save_agent_config(data.name, config); // TODO save config in agent flow
      }
    }
  }

  function deleteNode() {
    nodes.update((nodes) => {
      nodes = nodes.filter((node) => node.id !== id);
      return nodes;
    });
  }
</script>

<div>
  {#if data.properties}
    {@const name = data.name}
    {@const props = data.properties}
    <Card padding="none">
      <div class="flex justify-between items-center pl-4 pr-0 mb-2">
        <h3 class="text-xl pt-2">{data.schema?.["title"] || name}</h3>
        <Button onclick={deleteNode}><CloseOutline /></Button>
      </div>
      <h4 class="text-sm pl-4 pb-4">{data.schema?.["description"] || ""}</h4>
      <form class="grid grid-cols-6 gap-4 p-4">
        <Toggle checked={data.enabled as boolean} onchange={() => toggle_agent()} class="col-span-6"
        ></Toggle>
        {#each props.keys() as key}
          {@const prop = props.get(key)}
          {#if prop}
            <Label class="col-span-6 space-y-2">
              <h3>{prop.title || key}</h3>
              <p class="text-xs text-gray-500">{prop["description"]}</p>
              {#if prop.type === "boolean"}
                <Toggle
                  bind:checked={() => prop.value, (value) => props.set(key, { ...prop, value })}
                />
              {:else if prop.type === "integer"}
                <NumberInput
                  bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                />
              {:else if prop.type === "number"}
                <Input
                  type="text"
                  bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                />
              {:else if prop.type === "string" || prop.type === "string?"}
                <Input
                  type="text"
                  bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                />
              {:else if prop.type === "string[]"}
                <Textarea
                  bind:value={() => prop.value, (value) => props.set(key, { ...prop, value })}
                  rows={4}
                />
              {:else}
                <Input type="text" value={prop.value} disabled />
              {/if}
            </Label>
          {/if}
        {/each}
        <Button onclick={() => save()} class="w-fit" outline>Save</Button>
      </form>
    </Card>
  {:else}
    {@const name = data.name}
    <Card padding="none">
      <div class="flex justify-between items-center pl-4 pr-0 mb-2">
        <h3 class="text-xl pt-2">{name}</h3>
        <Button onclick={deleteNode}><CloseOutline /></Button>
      </div>
      <form class="grid grid-cols-6 gap-4 p-4">
        <Toggle checked={data.enabled as boolean} onchange={() => toggle_agent()} class="col-span-6"
        ></Toggle>
        {#if data.config}
          {@const config = data.config as Record<string, any>}
          {#each Object.keys(config) as key}
            <Label class="col-span-6 space-y-2">
              <span>{key}</span>
              {#if typeof config[key] === "boolean"}
                <Toggle checked={config[key]} disabled />
              {:else}
                <Input type="text" value={config[key]} disabled />
              {/if}
            </Label>
          {/each}
          <Button class="w-fit" outline disabled>Save</Button>
        {/if}
      </form>
    </Card>
  {/if}
</div>
