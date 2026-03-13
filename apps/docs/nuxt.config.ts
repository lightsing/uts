// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@nuxt/eslint',
    '@nuxt/image',
    '@nuxt/ui',
    '@nuxt/content',
    'nuxt-og-image',
  ],

  devtools: {
    enabled: true,
  },

  css: ['~/assets/css/main.css'],

  site: {
    url: 'https://docs.timestamps.now',
    name: 'UTS Documentation',
    description:
      'Universal Timestamps - Decentralized timestamping protocol with EAS attestations',
  },

  content: {
    build: {
      markdown: {
        toc: {
          searchDepth: 2,
          depth: 3,
        },
        highlight: {
          langs: [
            'typescript',
            'javascript',
            'rust',
            'bash',
            'json',
            'yaml',
            'toml',
            'solidity',
          ],
        },
      },
    },
  },

  experimental: {
    asyncContext: true,
  },

  compatibilityDate: '2024-07-11',

  nitro: {
    prerender: {
      routes: ['/'],
      crawlLinks: true,
      autoSubfolderIndex: false,
    },
  },

  eslint: {
    config: {
      stylistic: {
        commaDangle: 'never',
        braceStyle: '1tbs',
      },
    },
  },

  icon: {
    provider: 'iconify',
  },
})
