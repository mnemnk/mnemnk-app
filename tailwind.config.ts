import { join } from 'path';
import type { Config } from 'tailwindcss';
import forms from '@tailwindcss/forms';

import { skeleton } from '@skeletonlabs/tw-plugin';
import flowbitePlugin from 'flowbite/plugin'

const config = {
	// Opt for dark mode to be handled via the class method
	darkMode: 'class',
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		join(require.resolve(
			'@skeletonlabs/skeleton'),
			'../**/*.{html,js,svelte,ts}'
		),
		'./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'
	],
	theme: {
		extend: {},
	},
	plugins: [
		forms,
		skeleton({
		themes: { preset: [ "wintry" ] }
		}),
		flowbitePlugin,
	]
} satisfies Config;

export default config;
