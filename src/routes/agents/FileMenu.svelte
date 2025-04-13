<script lang="ts" module>
  const MENU_ITEM_CLASS =
    "w-full text-left text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400";

  interface Props {
    onNewFlow: () => void;
    onRenameFlow: () => void;
    onDeleteFlow: () => void;
    onSaveFlow: () => void;
    onExportFlow: () => void;
    onImportFlow: () => void;
  }
</script>

<script lang="ts">
  import { Accordion, AccordionItem } from "flowbite-svelte";

  let { onNewFlow, onRenameFlow, onDeleteFlow, onSaveFlow, onExportFlow, onImportFlow }: Props =
    $props();

  let open = $state(false);

  function handleNew() {
    open = false;
    onNewFlow();
  }

  function handleRename() {
    open = false;
    onRenameFlow();
  }

  function handleDelete() {
    open = false;
    onDeleteFlow();
  }

  function handleSave() {
    open = false;
    onSaveFlow();
  }

  function handleExport() {
    open = false;
    onExportFlow();
  }

  function handleImport() {
    open = false;
    onImportFlow();
  }
</script>

<div class="absolute top-0 left-45 w-40 z-10 backdrop-blur-xs">
  <Accordion flush>
    <AccordionItem
      bind:open
      borderBottomClass="border-b group-last:border-none"
      paddingFlush="pl-2 pr-2 py-1"
      transitionType="slide"
      transitionParams={{ duration: 200 }}
    >
      <div slot="header">
        <h4 class="text-left">File</h4>
      </div>
      <button type="button" class={MENU_ITEM_CLASS} onclick={handleNew}>New</button>
      <button type="button" class={MENU_ITEM_CLASS} onclick={handleRename}>Rename</button>
      <button type="button" class={MENU_ITEM_CLASS} onclick={handleDelete}>Delete</button>
      <button type="button" class="{MENU_ITEM_CLASS} border-b border-gray-700" onclick={handleSave}
        >Save</button
      >
      <button type="button" class={MENU_ITEM_CLASS} onclick={handleExport}>Export</button>
      <button type="button" class={MENU_ITEM_CLASS} onclick={handleImport}>Import</button>
    </AccordionItem>
  </Accordion>
</div>
