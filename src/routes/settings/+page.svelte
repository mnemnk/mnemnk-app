<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";

  import { Button } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { get_settings_filepath } from "@/lib/utils";

  import Core from "./Core.svelte";

  const { data } = $props();
  const settings = $derived(data.settings);

  async function open_settings_file() {
    const path = await get_settings_filepath();
    await open(path);
  }

  async function reindex_text() {
    await invoke("reindex_text_cmd");
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="flex">
    <h1 class="text-xl font-semibold sm:text-2xl">Settings</h1>
    <Button onclick={open_settings_file} class="ml-auto">config.yml</Button>
  </div>
  <Core {settings} />

  <Card title="Search">
    <Button onclick={reindex_text} class="w-fit m-2" outline>Reindex Text</Button>
  </Card>
</main>
