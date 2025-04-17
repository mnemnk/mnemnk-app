<script lang="ts" module>
  interface Message {
    type: string;
    data: {
      content: string | string[];
    };
  }

  interface Props {
    messages: Message | Message[];
  }
</script>

<script lang="ts">
  import { Avatar, Card } from "flowbite-svelte";

  let { messages }: Props = $props();

  let msgs = $derived(Array.isArray(messages) ? messages : messages ? [messages] : []);
</script>

<div class="nodrag nowheel max-h-[800px] overflow-y-auto">
  {#each msgs as message}
    <Card class="mb-1 min-w-full">
      <div class="flex items-center space-x-4 rtl:space-x-reverse">
        <Avatar class="flex-none shrink-0">{message.type}</Avatar>
        <div class="grow">
          <pre class="text-sm text-gray-500 dark:text-gray-400 text-wrap">{message.data
              ?.content}</pre>
        </div>
      </div>
    </Card>
  {/each}
</div>
