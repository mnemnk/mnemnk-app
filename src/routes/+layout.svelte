<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  import Attribution from "@/components/Attribution.svelte";

  import "../app.css";

  /** @type {{children?: import('svelte').Snippet}} */
  let { children } = $props();

  async function on_keydown(event: KeyboardEvent) {
    if (event.repeat) {
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      await getCurrentWindow().close();
    }
  }
</script>

{@render children?.()}
<Attribution />

<svelte:window on:keydown={on_keydown} />
