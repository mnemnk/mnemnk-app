<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import hotkeys from "hotkeys-js";

  import { goto } from "$app/navigation";

  import Attribution from "@/components/Attribution.svelte";
  import NavBar from "@/components/NavBar.svelte";

  import "../app.css";

  const { children, data } = $props();

  const key_close = "Escape";
  const key_fullscreen = $derived(data.settings.shortcut_keys["fullscreen"]);
  const key_search = $derived(data.settings.shortcut_keys["search"]);

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

  // async function on_keydown(event: KeyboardEvent) {
  //   if (event.repeat) {
  //     return;
  //   }
  //   console.log(event.key);
  //   if (event.key === "Escape") {
  //     event.preventDefault();
  //     await getCurrentWindow().close();
  //   } else if (event.key === key_fullscreen) {
  //     event.preventDefault();
  //     if (await getCurrentWindow().isFullscreen()) {
  //       await getCurrentWindow().setFullscreen(false);
  //     } else {
  //       await getCurrentWindow().setFullscreen(true);
  //     }
  //   } else if (event.key === key_search) {
  //     event.preventDefault();
  //     goto("/search");
  //   }
  // }
</script>

<NavBar />
{@render children?.()}
<Attribution />

<!-- <svelte:window on:keydown={on_keydown} /> -->
