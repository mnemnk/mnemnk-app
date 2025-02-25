<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import Attribution from "@/components/Attribution.svelte";
  import NavBar from "@/components/NavBar.svelte";

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
    } else if (event.key === "F11") {
      event.preventDefault();
      if (await getCurrentWindow().isFullscreen()) {
        await getCurrentWindow().setFullscreen(false);
      } else {
        await getCurrentWindow().setFullscreen(true);
      }
    }
  }
</script>

<NavBar />
{@render children?.()}
<Attribution />

<svelte:window on:keydown={on_keydown} />
