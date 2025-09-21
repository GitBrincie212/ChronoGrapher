// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

import tailwindcss from '@tailwindcss/vite';

import solidJs from '@astrojs/solid-js';

// https://astro.build/config
export default defineConfig({
  integrations: [starlight({
      title: 'Chronographer',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/withastro/starlight' }],
      sidebar: [
          {
              label: 'Guides',
              items: [
                  { label: 'Example Guide', slug: 'guides/example' },
              ],
          },
          {
              label: 'Reference',
              autogenerate: { directory: 'reference' },
          },
      ],
  }), solidJs()],

  vite: {
    plugins: [tailwindcss()],
    build: {
        assetsDir: 'assets',
    },
  },
});