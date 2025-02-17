import forms from "@tailwindcss/forms";
import flowbitePlugin from "flowbite/plugin";
import type { Config } from "tailwindcss";

const config = {
  // Opt for dark mode to be handled via the class method
  darkMode: "class",
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}",
  ],
  theme: {
    extend: {},
  },
  plugins: [forms, flowbitePlugin],
} satisfies Config;

export default config;
