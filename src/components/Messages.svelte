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
  import DOMPurify from "dompurify";
  import { Avatar, Card } from "flowbite-svelte";
  import { marked } from "marked";

  let { messages }: Props = $props();

  let msgs = $derived.by(() => {
    let msgArray = Array.isArray(messages) ? messages : messages ? [messages] : [];
    return msgArray.map((msg) => {
      if (msg.type === "ai") {
        if (typeof msg.data.content === "string") {
          let html = marked.parse(DOMPurify.sanitize(msg.data.content));
          return { type: msg.type, html };
        } else if (Array.isArray(msg.data.content)) {
          let html = marked.parse(DOMPurify.sanitize(msg.data.content.join("\n\n")));
          return { type: msg.type, html };
        }
      } else {
        if (typeof msg.data.content === "string") {
          let html = msg.data.content;
          return { type: msg.type, html };
        } else if (Array.isArray(msg.data.content)) {
          let html = msg.data.content.join("\n\n");
          return { type: msg.type, html };
        }
      }
      return { type: msg.type, html: "" };
    });
  });
</script>

<div class="nodrag nowheel max-h-[800px] overflow-y-auto">
  {#each msgs as message}
    <Card class="mb-1 min-w-full">
      <div class="flex items-center space-x-4 rtl:space-x-reverse">
        <Avatar class="flex-none shrink-0">{message.type}</Avatar>
        <div class="grow">
          {#if message.type === "ai"}
            {@html message.html}
          {:else}
            {message.html}
          {/if}
        </div>
      </div>
    </Card>
  {/each}
</div>
