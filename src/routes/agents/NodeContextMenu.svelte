<script lang="ts" module>
  const MENU_CLASS =
    "w-full text-left pl-2 mb-1 text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400";
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
  class="absolute w-16 p-0 m-0 z-10 text-lg bg-transparent backdrop-blur-xs border-gray-300 dark:border-gray-800 border-solid border-1 shadow-md"
>
  <button onclick={handleStart} class={MENU_CLASS}>Start</button>
  <button onclick={handleStop} class="{MENU_CLASS} border-b border-gray-600 pb-1 mb-1">Stop</button>
  <button onclick={handleCut} class={MENU_CLASS}>Cut</button>
  <button onclick={handleCopy} class="{MENU_CLASS} pb-1">Copy</button>
</div>
