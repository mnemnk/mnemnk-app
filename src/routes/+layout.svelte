<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { setContext } from "svelte";

  import hotkeys from "hotkeys-js";

  import { goto } from "$app/navigation";

  import Attribution from "@/components/Attribution.svelte";
  import NavBar from "@/components/NavBar.svelte";

  import "../app.css";
  import type { LayoutProps } from "./$types";

  const { children, data }: LayoutProps = $props();

  setContext("coreSettings", data.coreSettings);
  setContext("agentFlows", () => data.agentFlows);

  const key_close = "Escape";
  const key_fullscreen = $derived(data.coreSettings.shortcut_keys["fullscreen"]);
  const key_search = $derived(data.coreSettings.shortcut_keys["search"]);

  $effect(() => {
    hotkeys(key_close, () => {
      getCurrentWindow().close();
    });
    hotkeys(key_fullscreen, () => {
      getCurrentWindow()
        .isFullscreen()
        .then((isFullscreen) => {
          if (isFullscreen) {
            getCurrentWindow().setFullscreen(false);
          } else {
            getCurrentWindow().setFullscreen(true);
          }
        });
    });
    hotkeys(key_search, () => {
      goto("/search");
    });

    return () => {
      hotkeys.unbind(key_fullscreen);
      hotkeys.unbind(key_search);
    };
  });
</script>

{#if data.coreSettings.mnemnk_dir}
  <NavBar />
{/if}
{@render children?.()}
<Attribution />
