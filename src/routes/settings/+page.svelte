<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  import { Button } from "flowbite-svelte";

  import { get_settings_filepath } from "@/lib/utils";

  import Core from "./Core.svelte";

  let { data } = $props();

  let core_settings = $state(data.settings.core);

  async function open_settings_file() {
    let path = await get_settings_filepath();
    await open(path);
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div class="flex">
    <h1 class="text-xl font-semibold sm:text-2xl">Settings</h1>
    <Button onclick={open_settings_file} class="ml-auto">config.yml</Button>
  </div>
  <Core settings={core_settings} />
</main>
