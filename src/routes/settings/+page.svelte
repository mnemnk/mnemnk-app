<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  import { preventDefault } from "svelte/legacy";

  let { data } = $props();

  let settings = $state(data.settings);

  async function save() {
    let result = await invoke("set_settings_json", { jsonStr: settings });
    console.log(result);
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <div>
    <form onsubmit={preventDefault(save)}>
      <textarea bind:value={settings} class="w-full h-96 dark:!bg-gray-800"></textarea>
      <button type="submit" class="btn">Save</button>
    </form>
  </div>
</main>
