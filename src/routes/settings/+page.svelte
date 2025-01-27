<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  import { preventDefault } from "svelte/legacy";

  import { AppBar } from "@skeletonlabs/skeleton";
  import IconsMdiChevronLeft from "~icons/mdi/chevron-left";

  interface Props {
    data: any;
  }

  let { data }: Props = $props();

  let settings = $state(data.settings);

  async function save() {
    let result = await invoke("set_settings_json", { jsonStr: settings });
    console.log(result);
  }
</script>

<div>
  <AppBar class="!bg-transparent">
    {#snippet lead()}
      <a href="/"><IconsMdiChevronLeft /></a>
    {/snippet}
    <div class="text-xl">Settings</div>
  </AppBar>
  <main class="container mx-auto p-8 space-y-8">
    <div>
      <form onsubmit={preventDefault(save)}>
        <textarea bind:value={settings} class="w-full h-96 !bg-surface-500"></textarea>
        <button type="submit" class="btn">Save</button>
      </form>
    </div>
  </main>
</div>
