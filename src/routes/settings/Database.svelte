<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ask, open, save, message } from "@tauri-apps/plugin-dialog";

  import { Button, Label, NumberInput, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { exitApp, setCoreSettings } from "@/lib/utils";

  interface Props {
    settings: Record<string, any>;
  }

  const { settings }: Props = $props();

  let backup_interval_hours = $state(settings["backup_interval_hours"]);
  let max_backup_count = $state(settings["max_backup_count"]);
  let enable_auto_backup = $state(settings["enable_auto_backup"]);

  async function saveSettings() {
    await setCoreSettings({
      backup_interval_hours,
      max_backup_count,
      enable_auto_backup,
    });
    // confirm restart
    await message("Mnemnk will quit to apply changes.\n\nPlease restart.");
    await exitApp();
  }

  async function handleExport() {
    try {
      // Open save dialog
      const filePath = await save({
        filters: [
          {
            name: "JSONL",
            extensions: ["jsonl"],
          },
        ],
        defaultPath: "mnemnk_backup.jsonl",
      });

      if (!filePath) return; // User cancelled

      // Call export command
      await invoke("export_events_cmd", { path: filePath });

      await message("Events exported successfully!", {
        title: "Export Successful",
        kind: "info",
      });
    } catch (error) {
      console.error("Export failed:", error);
      await message(`Export failed: ${error}`, {
        title: "Export Failed",
        kind: "error",
      });
    }
  }

  async function handleImport() {
    try {
      // Open file dialog
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "JSONL",
            extensions: ["jsonl"],
          },
        ],
      });

      if (!selected) return; // User cancelled

      // Confirm import
      const confirmed = await ask(
        "Importing will merge with existing records. This operation cannot be undone. Continue?",
        {
          title: "Confirm Import",
          kind: "warning",
          okLabel: "Import",
          cancelLabel: "Cancel",
        },
      );

      if (!confirmed) return;

      // Call import command
      await invoke("import_events_cmd", { path: selected });

      await message("Events imported successfully!", {
        title: "Import Successful",
        kind: "info",
      });
    } catch (error) {
      console.error("Import failed:", error);
      await message(`Import failed: ${error}`, {
        title: "Import Failed",
        kind: "error",
      });
    }
  }

  async function handleReindexText() {
    try {
      await message("Reindexing text data. This may take a while...", {
        title: "Reindexing",
        kind: "info",
      });

      await invoke("reindex_text_cmd");

      await message("Text reindexing completed!", {
        title: "Reindexing Complete",
        kind: "info",
      });
    } catch (error) {
      console.error("Reindexing failed:", error);
      await message(`Reindexing failed: ${error}`, {
        title: "Reindexing Failed",
        kind: "error",
      });
    }
  }
</script>

<Card title="Database" tooltip="Database management functions">
  <form class="grid grid-cols-6 gap-6 mb-6">
    <Toggle bind:checked={enable_auto_backup}>Enable Auto Backup</Toggle>

    <Label class="col-span-6 space-y-2">
      <span>Backup Interval (hours)</span>
      <NumberInput min="1" bind:value={backup_interval_hours} placeholder="24" />
    </Label>

    <Label class="col-span-6 space-y-2">
      <span>Max Backup Count</span>
      <NumberInput min="1" bind:value={max_backup_count} placeholder="7" />
    </Label>

    <Button class="w-fit" onclick={saveSettings} outline>Save</Button>
  </form>

  <div class="flex flex-col gap-4">
    <div>
      <Button color="alternative" onclick={handleExport}>Export Events</Button>
    </div>

    <div>
      <Button color="alternative" onclick={handleImport}>Import Events</Button>
    </div>

    <div>
      <Button color="alternative" onclick={handleReindexText}>Reindex Text</Button>
      <p class="ml-4">Rebuild the search index for all text content.</p>
    </div>
  </div>
</Card>
