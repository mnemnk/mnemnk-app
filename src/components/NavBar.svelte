<script>
  import { BottomNav, BottomNavItem, Tooltip } from "flowbite-svelte";
  import { CogOutline, HomeOutline, SearchOutline, UsersOutline } from "flowbite-svelte-icons";

  import { page } from "$app/state";

  let activeUrl = $derived(page.url.pathname);
</script>

<BottomNav
  {activeUrl}
  position="fixed"
  navType="application"
  classActive="bg-surgace-200 dark:bg-surface-800"
  classOuter="top-3 h-6 opacity-70"
  classInner="grid-cols-4"
>
  <BottomNavItem btnName="Home" href="/" appBtnPosition="left">
    <HomeOutline
      class="w-4 h-4 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500 focus:outline-hidden"
      tabindex={-1}
    />
    <Tooltip arrow={false}>Home</Tooltip>
  </BottomNavItem>
  <BottomNavItem btnName="Agents" href="/agents" appBtnPosition="middle">
    <UsersOutline
      class="w-4 h-4 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500 focus:outline-hidden"
      tabindex={-1}
    />
    <Tooltip arrow={false}>Agents</Tooltip>
  </BottomNavItem>
  <BottomNavItem btnName="Search" href="/search" appBtnPosition="middle">
    <SearchOutline
      class="w-4 h-4 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500 focus:outline-hidden"
      tabindex={-1}
    />
    <Tooltip arrow={false}>Search</Tooltip>
  </BottomNavItem>
  <BottomNavItem btnName="Settings" href="/settings" appBtnPosition="right">
    <CogOutline
      class="w-4 h-4 mb-1/2 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500 focus:outline-hidden"
      tabindex={-1}
    />
    <Tooltip arrow={false}>Settings</Tooltip>
  </BottomNavItem>
</BottomNav>
