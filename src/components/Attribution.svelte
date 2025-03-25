<script>
  import { getName, getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-shell";

  function onclick() {
    open("https://github.com/mnemnk/mnemnk-app");
  }
</script>

<div class="fixed bottom-1 right-1 z-50 p-0 text-xs text-gray-400">
  <button {onclick}>
    {#await getName() then name}
      {name}
    {/await}
    {#await getVersion() then version}
      {version}
    {/await}
  </button>
</div>
