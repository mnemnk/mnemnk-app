<script lang="ts" module>
  interface Message {
    type?: string;
    role?: string;
    content?: string | string[];
    data?: {
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
      let role = msg.type || msg.role || "user";
      if (role === "assistant") {
        role = "ai";
      }
      let content = msg.data?.content || msg.content;
      if (role === "ai") {
        if (typeof content === "string") {
          let html = marked.parse(DOMPurify.sanitize(content));
          return { role, html };
        } else if (Array.isArray(content)) {
          let html = marked.parse(DOMPurify.sanitize(content.join("\n\n")));
          return { role, html };
        }
      } else {
        if (typeof content === "string") {
          let html = content;
          return { role, html };
        } else if (Array.isArray(content)) {
          let html = content.join("\n\n");
          return { role, html };
        }
      }
      return { role, html: "" };
    });
  });
</script>

<div class="nodrag nowheel max-h-[800px] overflow-y-auto">
  {#each msgs as message}
    <Card class="mb-1 min-w-full">
      <div class="flex items-center space-x-4 rtl:space-x-reverse">
        <Avatar class="flex-none shrink-0">{message.role}</Avatar>
        <div class="grow">
          {#if message.role === "ai"}
            {@html message.html}
          {:else}
            {message.html}
          {/if}
        </div>
      </div>
    </Card>
  {/each}
</div>
