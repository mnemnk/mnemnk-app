<script lang="ts" module>
  const MENU_CLASS = "w-full text-left pl-2 hover:bg-gray-700";
</script>

<script lang="ts">
  type Props = {
    x: number;
    y: number;
    hideNodeContextMenu: () => void;
    onstart: () => void;
    onstop: () => void;
    oncut: () => void;
    oncopy: () => void;
  };

  let { x, y, hideNodeContextMenu, onstart, onstop, oncut, oncopy }: Props = $props();

  function handleStart() {
    hideNodeContextMenu();
    onstart();
  }

  function handleStop() {
    hideNodeContextMenu();
    onstop();
  }

  function handleCut() {
    hideNodeContextMenu();
    oncut();
  }

  function handleCopy() {
    hideNodeContextMenu();
    oncopy();
  }
</script>

<div
  style="top: {y}px; left: {x}px;"
  class="absolute p-0 m-0 z-10 text-lg fg-black bg-white dark:fg-white dark:bg-gray-800 border-gray-300 dark:border-gray-700 border-solid border-2 shadow-md"
>
  <button onclick={handleStart} class={MENU_CLASS}>Start</button>
  <button onclick={handleStop} class={MENU_CLASS}>Stop</button>
  <hr class="my-2" />
  <button onclick={handleCut} class={MENU_CLASS}>Cut</button>
  <button onclick={handleCopy} class="{MENU_CLASS} pb-1">Copy</button>
</div>
