<script lang="ts">
  import { Button, Input, Label, NumberInput, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { set_core_settings } from "@/lib/utils";

  interface Props {
    settings: Record<string, any>;
  }

  const { settings }: Props = $props();

  let autostart = $state(settings["autostart"]);
  let data_dir = $state(settings["data_dir"]);
  let shortcut_key = $state(settings["shortcut_key"]);
  let shortcut_keys = $state(settings["shortcut_keys"]);
  let thumbnail_width = $state(settings["thumbnail_width"]);
  let thumbnail_height = $state(settings["thumbnail_height"]);

  async function save_settings() {
    await set_core_settings({
      autostart,
      data_dir,
      shortcut_key,
      shortcut_keys,
      thumbnail_width,
      thumbnail_height,
    });
  }
</script>

<Card title="Core">
  <form class="grid grid-cols-6 gap-6">
    <Label class="col-span-6 space-y-2">
      <span>Data Directory</span>
      <Input type="text" placeholder="Data Directory" bind:value={data_dir} />
    </Label>
    <Toggle bind:checked={autostart}>Auto Start</Toggle>
    <Label class="col-span-6 space-y-2">
      <span>Shortcut Key</span>
      <Input type="text" bind:value={shortcut_key} />
    </Label>
    <Label class="col-span-3 space-y-2">
      <span>Thumbnail Width</span>
      <NumberInput bind:value={thumbnail_width} />
    </Label>
    <Label class="col-span-3 space-y-2">
      <span>Height</span>
      <NumberInput bind:value={thumbnail_height} />
    </Label>
    <Button onclick={save_settings} class="w-fit" outline>Save</Button>
  </form>
</Card>
