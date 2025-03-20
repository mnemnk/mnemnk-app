<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { message, open } from "@tauri-apps/plugin-dialog";

  import { Button, ButtonGroup, Input, Label, NumberInput, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { exit_app, set_core_settings } from "@/lib/utils";

  interface Props {
    settings: Record<string, any>;
  }

  const { settings }: Props = $props();

  let autostart = $state(settings["autostart"]);
  let mnemnk_dir = $state(settings["mnemnk_dir"]);
  let shortcut_keys = $state(settings["shortcut_keys"]);
  let thumbnail_width = $state(settings["thumbnail_width"]);
  let thumbnail_height = $state(settings["thumbnail_height"]);
  let day_start_hour = $state(settings["day_start_hour"]);

  async function openMnemnkDir() {
    const dir = await open({ directory: true });
    if (dir) {
      mnemnk_dir = dir;
    }
  }

  async function reindexYMD() {
    await invoke("reindex_ymd_cmd");
  }

  async function saveSettings() {
    await set_core_settings({
      autostart,
      mnemnk_dir,
      shortcut_keys,
      thumbnail_width,
      thumbnail_height,
      day_start_hour,
    });
    // confirm restart
    await message("Mnemnk will quit to apply changes.\n\nPlease restart.");
    await exit_app();
  }
</script>

<Card title="Core">
  <form class="grid grid-cols-6 gap-6">
    <Label class="col-span-6 space-y-2">
      <span>Mnemnk Directory</span>
      <ButtonGroup class="w-full grid grid-cols-6 gap-0">
        <Input
          class="col-span-5"
          color={mnemnk_dir ? "base" : "red"}
          type="text"
          placeholder="Please specify Mnemnk base directory"
          bind:value={mnemnk_dir}
        />
        <Button
          color={mnemnk_dir ? "primary" : "red"}
          outline={!!mnemnk_dir}
          onclick={openMnemnkDir}>Open</Button
        >
      </ButtonGroup>
    </Label>

    <Toggle bind:checked={autostart}>Auto Start</Toggle>

    <div class="col-span-6">
      <h3 class="text-lg font-semibold">Shortcut Keys</h3>
    </div>

    <Label class="col-span-2 space-y-2">
      <span>Global Shortcut</span>
    </Label>
    <Input class="col-span-4" type="text" bind:value={shortcut_keys["global_shortcut"]} />

    <Label class="col-span-2 space-y-2">
      <span>Fullscreen</span>
    </Label>
    <Input class="col-span-4" type="text" bind:value={shortcut_keys["fullscreen"]} />

    <Label class="col-span-2 space-y-2">
      <span>Screenshot Only</span>
    </Label>
    <Input class="col-span-4" type="text" bind:value={shortcut_keys["screenshot_only"]} />

    <Label class="col-span-2 space-y-2">
      <span>Search</span>
    </Label>
    <Input class="col-span-4" type="text" bind:value={shortcut_keys["search"]} />

    <Label class="col-span-3 space-y-2">
      <span>Thumbnail Width</span>
      <NumberInput bind:value={thumbnail_width} />
    </Label>
    <Label class="col-span-3 space-y-2">
      <span>Height</span>
      <NumberInput bind:value={thumbnail_height} />
    </Label>

    <Label class="col-span-6 space-y-2">
      <span>Day Start Hour</span>
      <div class="grid grid-cols-6 gap-6">
        <NumberInput bind:value={day_start_hour} class="col-span-5" />
        <Button onclick={reindexYMD} outline>Reindex YMD</Button>
      </div>
    </Label>

    <Button disabled={!mnemnk_dir} onclick={saveSettings} class="w-fit" outline>Save</Button>
  </form>
</Card>
